# MFastAPI × Mojo × RMojo 対応設計(2026-11-23)

`DESIGN.md`(2026-11-14、段階A/B/C)の続きとして、**Mojo 対応を「公式 Mojo」と「RMojo」の2系統でどう持つか**を決める設計書。
実装の現状は `CLAUDE.md`・`PORTING.md` が正本で、この文書は**これから作るものの設計**。推測で「できた」と書かない(未検証は未検証と書く)。

## 1. 現状(コードで確認した事実)

| 部品 | 現状 |
|---|---|
| `python/mfastapi/mojo.py` | `mojo_endpoint(fn, params, returns)`。Mojo(など)の関数に、宣言したシグネチャを持つ薄い Python 関数をかぶせ、FastAPI に登録する。検証・DI・OpenAPI は本家のまま。Mojo 関数の実行中は GIL を保持する。 |
| `mojo/*.mojo` | 公式 Mojo 製。`mfastapi_cabi.mojo`(C ABI、ctypes 用)、`mfastapi_native.mojo`(`PythonModuleBuilder` 拡張)、`mfastapi_match/table`(Starlette 互換のパス照合・trie)、`server_b/c.mojo`(純 Mojo サーバー=基準実装)。 |
| `rust/mfastapi-server` | 配布の正。GET/HEAD/POST/PUT/PATCH/DELETE と JSON 本文、422/405/413、OpenAPI。純 Mojo 版は「正しい Mojo として動くかの確認」用に残す(2026-11-16 決定)。 |
| `rust/mfastapi-core` / `mfastapi-py` | IR・OpenAPI・パス照合の Rust コアと PyO3 拡張。 |
| RMojo(`F:\RMojo`、v0.4.4) | Rust 製の Mojo 風言語。**単一バイナリ `rmojo`(`src/main.rs` 3037 行、ライブラリ化されていない)**。Python 構文のサブセット+`.mojo` の `fn/var/struct` 等(`src/mojo_syntax.rs`)。型付き `int -> int` 関数だけをコンパイル(CPython 3.14 比 約1.6〜1.9倍)、型なしは CPython より遅い。整数は 64bit(溢れはエラー)。Windows/Linux/macOS/Android/Android TV(エミュ)で動作確認済み、iOS はコンパイル確認のみ。**import・ポインタ・パラメータ化・スライス・タプル型は未対応**。 |

つまり、いま「RMojo で FastAPI のハンドラを動かす」道は**無い**(RMojo はコマンドとしてしか呼べない)。これを作るのが本設計の中心。

## 2. なぜ2系統か(役割分担)

| | 公式 Mojo | RMojo |
|---|---|---|
| 強み | 本物の Mojo。SIMD・GPU・MAX。ホットパスの最高速。 | Rust 製で**公式 Mojo が動かない所(ネイティブ Windows、Android、iOS、TV、BSD)でも動く**。メモリ安全。import・ファイル・ネットワークを持たないので**利用者が書いたハンドラの隔離実行**に向く。 |
| 弱み | ベータ。構文が版で変わる。ネイティブ Windows 未対応。呼び出しは GIL を保持。 | 言語のサブセット。型なしは遅い。SIMD は通常計算。 |
| MFastAPI での役割 | 計算ホットパス拡張(`.so`)と、基準実装(純 Mojo サーバー) | **どこでも動く既定のハンドラ実行系**、段階的に Mojo 互換を広げる |

方針: **同じハンドラ(Mojo 風ソース)が、公式 Mojo でも RMojo でも同じ結果を返す**ことを、互換の客観指標にする(既存 `tests/mojo/parity.py` の延長)。

## 3. ハンドラ契約 `mfastapi.handler.v1`

バックエンドが何であっても、ハンドラは次の契約で呼ばれる(段階A/B/C共通の IR に、実行系の種類を1項目足す)。

- **入力**: IR のパラメータ(パス/クエリ/本文)を検証済みの値にした列。値の型は `int(i64)`・`float(f64)`・`bool`・`str`・`list[…]`・`dict[str,…]`・`null`。本文 JSON は RS-JSON の上限(サイズ・深さ・重複キー)を通したものだけ。
- **出力**: 上と同じ型の値、または `error{status, detail}`。
- **失敗の写像**: ハンドラの例外(Mojo の `raise`、RMojo のエラー)→ HTTP 400(既定。IR で変更可)、実行時間超過 → 504、メモリ/ステップ上限超過 → 503。
- **純粋性**: ハンドラは外部状態に触れない(RMojo は構造上触れない。公式 Mojo 側は規約とレビューで担保)。副作用が要る処理は Python/Rust 側に残す。
- **IR の追加項目**: `runtime: "python" | "mojo" | "rmojo" | "rust" | "cabi"`、`source`(パスまたはインライン)、`limits{steps, ms, mem_mb}`。

## 4. RMojo 側の作り(最初に必要な改造)

1. **ライブラリ化**: `src/main.rs` を `lib.rs`(`rmojo-core`)+薄い `main.rs` に分ける。公開 API は最小に:
   - `compile(source, kind: Py|Mojo) -> Result<Program, Diag>`
   - `Program::call(name, &[Value], &Limits) -> Result<Value, RunError>`
   - `Value` は §3 の型の和。
   これは機械的な分割だが 3000 行なので、**差分ゼロ(`tests/run.py` が CPython と一致)を保ったまま**段階的に行う。
2. **上限(Limits)**: ステップ数(命令カウンタ)・経過時間・メモリ(リスト/辞書/文字列の合計)・再帰の深さ。型付きコンパイル経路でもステップ上限を入れるか、コンパイル経路は「ループ回数上限つき」にする。**段階0の実測(2026-11-23、RMojo v0.4.4、`F:\RMojo	argeteleasemojo.exe`)**: 無限ループ(`while True`)とメモリ増加(`list.append` の無限ループ)は**5秒のタイムアウトまで止まらなかった**(上限なし)。再帰は `RecursionError: maximum recursion depth exceeded` で止まる(深さ上限は既にある)。よってステップ/メモリ上限は新規実装が必要。
3. **再入可能性**: `Program` は不変・`Send+Sync`、呼び出しごとの状態は呼び出し側に持つ。これでワーカー間で共有できる(1つのプロセスに1回だけ読み込む、CLAUDE.md の配備方針と整合)。
4. **`rmojo` コマンドは残す**(既存の配布・インストーラーは変えない)。

## 5. MFastAPI 側の作り

### 5.1 3つの入口(利用者が選ぶ)
- **A. 互換モード(本家 FastAPI のまま)**: `mojo_endpoint()` を拡張し、`runtime="rmojo"` なら PyO3 拡張(`rmojo-py`、`rust/mfastapi-py` に同居可)経由で `Program::call`。**GIL を手放す**(`py.allow_threads`)ので、公式 Mojo 拡張の「GIL 保持」の制限が消える。
- **B. Rust サーバー(配布の正)**: `rust/mfastapi-server` が `rmojo-core` を直接リンクし、IR の `runtime:"rmojo"` ルートを Python 無しで実行。ルート照合 → 検証 → `Program::call` → JSON 応答。ここが本命の速い経路。
- **C. 純 Mojo サーバー**: 従来どおり基準実装として維持(公式 Mojo の確認用)。

### 5.2 バックエンド選択(自動フォールバック)
`runtime="auto"` の順: `rust` → `mojo`(公式 Mojo がその OS で利用可で、`.so` が読めるとき)→ `rmojo` → `python`。選んだ実行系は起動時ログと `/openapi.json` の拡張項目 `x-mfastapi-runtime` に出す(隠さない)。公式 Mojo 版が無い OS(ネイティブ Windows 等)では自動的に RMojo になる。

### 5.3 型の対応表(OpenAPI ↔ IR ↔ ハンドラ)
`int`→`integer(int64)`、`float`→`number(double)`、`bool`→`boolean`、`str`→`string`、`list[T]`→`array`、`dict[str,T]`→`object`。64bit を超える整数は **422 で拒否**(RMojo の 64bit 制限と Pydantic の差が出る所なので、本家との差分テストに必ず入れる)。

## 6. Mojo 互換を広げる順番(RMojo)

ハンドラとして実用になる順(各段階で CPython/公式 Mojo と差分0を条件に進める):
1. **タプル型・スライス**(戻り値と入力の整形に必要)
2. **`def` の引数型検査**(現状は注釈のみ。IR の型と食い違うと危険)
3. **パラメータ化 `fn f[T: …]`・`struct S[T]`**(標準的な Mojo コードの多く)
4. **他のデコレーター(`@always_inline` 等は無視可、`@register_passable` は無視)**
5. **Mojo モジュールの import(`std` の一部のみ、許可リスト方式)** ← 隔離性を守るため、ファイル/ネットワークは入れない
6. SIMD の実命令化(open-cpu の検出結果を使う。**測って効く所だけ**)

「`trait` の適合検査」は後回し(いまはダックタイピングで動く。誤りを見逃す代わりに利用者のコードが通る)。

## 7. 検証計画(性能・互換の主張は実測のみ)

1. **差分テスト(最優先)**: 同じハンドラ集を ①CPython ②公式 Mojo ③RMojo で実行し、値が一致することを CI で確認。既存 `tests/mojo/parity.py` と `tests/mojo_endpoint/` を土台にする。OS は公式 Mojo が動く Linux/macOS と、RMojo のみの Windows/Android で分ける。
2. **HTTP 互換**: 本家 FastAPI と `/openapi.json`・応答・422 の形を比較(既存 `tests/compat`)。`runtime` を変えても応答が同一であること。
3. **上限の試験**: 無限ループ・巨大リスト・深い再帰が 504/503 で止まり、サーバーが生き続けること。
4. **境界費用**: 1リクエストあたりの `Program::call` 回数と、GIL 保持の有無(A 入口)での同時処理の差。
5. **性能**: fib/collatz/素数/JSON 変換を、本家(uvicorn)・Granian・MFastAPI 各 runtime で同一条件に測る。RMojo の「1.6〜1.9倍」は `int->int` のみの値で、HTTP 全体の倍率ではない。
6. **fuzz**: RMojo のパーサー(rustpython-parser)と `mojo_syntax` へ、壊れた入力・巨大入力。

## 8. 段階的な実装計画

| 段階 | 内容 | 完了条件 |
|---|---|---|
| 0 | RMojo に上限が効くかの試験(無限ループ・メモリ) | **済(2026-11-23)**: 無限ループ・メモリは上限なし、再帰のみ上限あり。段階1でステップ/メモリ上限を実装 |
| 1 | `rmojo-core` へライブラリ化+`Limits`+`Program::call` | `tests/run.py` 差分0、Rust 単体試験 |
| 2 | `rmojo-py`(PyO3)+`mojo_endpoint(runtime="rmojo")` | Linux/Windows で A 入口の差分テスト合格 |
| 3 | `mfastapi-server` から直接呼ぶ(B 入口)+`x-mfastapi-runtime` | CI のスモーク(起動して応答)に rmojo ルート追加 |
| 4 | `auto` フォールバックと OS 別 CI | Windows(公式 Mojo 無し)で自動的に rmojo になる試験 |
| 5 | §6 の互換拡張を1項目ずつ | 各項目で差分0 |
| 6 | Android 実機/TV での B 入口(サーバー内蔵アプリ) | 実機の動作記録(実機が無ければ「未確認」と書く) |

## 9. リスクと未確認

- **RMojo にステップ/メモリ上限が無い**(段階0で確認済み)。上限を入れるまでハンドラ実行系にしない(DoS になる)。
- **3000 行の単一ファイルのライブラリ化**で既存の差分試験が崩れるリスク → 段階1は「動作を変えない分割」に限る。
- **公式 Mojo の構文変更**(`fn`→`def` など)への追従は `version-control` の監視に頼る。RMojo は両方の綴りを受ける方針を維持。
- **64bit 制限**: Python の任意精度整数と食い違う → 422 で拒否し、OpenAPI に `int64` と明記。
- **iOS は RMojo もコンパイル確認止まり**。iOS の実機は未確認と書き続ける。
- **Milkha との重複**: 差別化は「本家アプリの段階移行」「RUNO 方針整合」「公式 Mojo が無い OS でも動く」。
- **REST 全廃方針**: ここで出す HTTP は FastAPI 互換の入口。MFastAPI/RMojo 自体の管理 API を新設する場合は Connect(`POST /<パッケージ>.<Service>/<Method>`)にする。新しい REST は作らない。
- **配備**: Mojo/RMojo ランタイムは VPS に1回だけ入れ、サイトごとに同梱しない。

## 10. 姉妹リポジトリとの関係(同時開発)

- `RMojo`: `rmojo-core` 化(段階1)。変更したら MFastAPI の差分テストと README/CLAUDE/PORTING を同日に更新。
- `MFastAPI`: 入口 A/B と IR 項目(段階2〜4)。
- `Reaxt`/`RPoRea`/`RPoem`/`open-cosmo`: RPoRea の C ABI から Mojo(公式)を呼ぶ検証は済み(`bindings/mojo`)。RMojo ハンドラを RPoRea の Connect メソッドとして登録する道は、段階3 の後で検討(同じ `Program::call` を `rporea_method` から呼ぶだけで済む見込み、未検証)。
- `pl2pl`: Python/Mojo 系の変換先として RMojo を選べるようにするのは、段階4 の後。

## 出典
- `DESIGN.md`(既存の調査結果と出典)
- RMojo の README/CLAUDE(`F:\RMojo`、v0.4.4 の記載)
- https://mojolang.org/docs/manual/python/mojo-from-python/
