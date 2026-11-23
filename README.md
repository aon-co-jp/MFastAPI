# MFastAPI

> **名前の変更(2026-11-15、互換性を壊す変更)**: 名称を `FastAPIM` から **`MFastAPI`** に改めた(旧 `mfastapi` → `FastAPIM` → `MFastAPI`)。Pythonのパッケージ名・import名は `mfastapi`、拡張モジュールは `mfastapi_rust` / `mfastapi_cext` / `mfastapi_native` / `mfastapi_handlers`、環境変数は `MFASTAPI_DISABLE`、C ABIの関数は `mfastapi_*`、GitHubは `aon-co-jp/MFastAPI`(旧URLは自動転送)。`fastapim` を使っていた場合は `mfastapi` に切り替える。v0.2.0までのRelease/タグの成果物名は `fastapim-*` のまま(以降のリリースから `mfastapi-*`)。PyPIへは公開していない。Windowsでは [RMojo](https://github.com/aon-co-jp/RMojo)(旧MojoW等を統合)の `compat/wsl-bridge` でWSL上の本物のMojoを使える。
>
> **Rename (2026-11-15, breaking change)**: renamed from `FastAPIM` to **`MFastAPI`**. Python package / import name `mfastapi`; extension modules `mfastapi_rust` / `mfastapi_cext` / `mfastapi_native` / `mfastapi_handlers`; kill switch `MFASTAPI_DISABLE`; C ABI `mfastapi_*`; GitHub `aon-co-jp/MFastAPI` (old URL redirects). If you used `fastapim`, switch to `mfastapi`. Artifacts of tags up to v0.2.0 keep the `fastapim-*` names. Not on PyPI. On Windows use `compat/wsl-bridge` of [RMojo](https://github.com/aon-co-jp/RMojo) (successor of MojoW) to drive the real Mojo in WSL.

> **対象 / Targets(目標であり、すべて検証済みではない / goals, not all verified)**: Python 2/3・Mojo・RMojo対応のFastAPI互換フレームワーク。見た目はPython構文、中身はRust。Windows・Linux・UNIX・macOS・TV(アプリのダウンロード機能がある場合)・Android・iPhone・iPad。**検証済み**: Python 3.9〜3.15のwheel(Windows/macOS/Linux)とFastAPI互換(差分テスト)。**未検証・未対応**: Python 2(Rust拡張のwheelは作れない。旧C版のみ過去に検証)、BSD等のUNIX、TV、Android/iOSの実機。 / Verified: wheels for Python 3.9-3.15 and FastAPI parity. Unverified: Python 2 (no Rust wheel; only the old C build was once tested), BSD/UNIX, TV, real Android/iOS devices.
>
> **互換性 / Compatibility**: FastAPIとMFastAPIの互換性は、実際のFastAPI(uvicorn)との差分テストで継続的に検証する(下記「互換性の確認方法と範囲」)。段階B・Cは検証済みの範囲(パス/クエリのint・str、422、OpenAPI)で応答が一致する。未対応の機能(Depends・Pydanticモデル本文・ミドルウェア等)は互換外として明記する。 / FastAPI-vs-MFastAPI compatibility is verified by differential tests against real FastAPI on uvicorn (see "互換性の確認方法と範囲" below). Stages B/C match FastAPI on the verified scope (int/str path and query params, 422 errors, OpenAPI); unsupported features (Depends, Pydantic body models, middleware) are explicitly out of scope.
>
> **アーキテクチャ(2026-11-14決定) / Architecture**: 見た目はPython構文(FastAPIと同じAPI)、中身・高速化・セキュリティは**Rust**。C++は採用しない。
> Python-looking surface (FastAPI's API); internals, speed and security in **Rust**. No C++.
> - `rust/mfastapi-core`: **Rust製コア**。Starlette互換のパス照合とコンパイル済みルート表(C版と同じC ABIのまま差し替え可能、`libmfastapi_core.so`)、Pydantic互換の整数解析・クエリ値の取り出し(長さ上限・i64範囲・安全なRust、`unsafe`はC ABIの薄い層だけ)。
> - 検証: ルート表はStarletteと48,000件で不一致0、速度はC版と同等(約1.4〜1.7µs/照合、`scripts/test-rust-table-wsl.sh`)。整数解析はPydanticと5,158件、クエリはparse_qslと15,030件で不一致0(`scripts/test-rust-wsl.sh`)。Pydanticの癖("0-5"が-5になる等)も再現した。
> - 以前のC版(`c/`)とMojo版(`mojo/`)は参照実装・比較用に残す(Mojoの段階B・Cサーバーは実験的)。Rust版が既定の方向。 / The C (`c/`) and Mojo (`mojo/`) versions stay as reference/comparison implementations; Rust is the default direction.
> - **Rust拡張 `mfastapi_rust`(PyO3)が `accelerate` の既定バックエンド**(`mfastapi.backend()=="rust"`)。`pip install mfastapi` で入り(setuptools-rust、ビルドできない環境では拡張なしで本家のまま動く)、Linux x86_64/ARM・Windows(64bit)・macOS(Intel/Apple silicon)のwheelで、バックエンドがrustであることと等価性テストをCIで確認する(Windows 32bitは対象外にした)。
> - **互換性の継続的な保証**: `.github/workflows/compat.yml` が push・PR のたびに Python 3.9/3.12/3.14 で、StarletteおよびFastAPIとの等価性テスト、Rustコアの単体テスト、Pydantic・`parse_qsl` との差分ファジングを実行し、不一致があれば失敗する。保証の範囲はルーティング(パス照合・パラメータ・405・リダイレクト・WebSocket)と、整数/クエリ値の解析。範囲外は上記の注記のとおり。
> - **HTTPサーバー層もRust(`rust/mfastapi-server`、hyper+tokio)**: Starlette互換ルート表でのルーティング、FastAPIと同一の422エラー、OpenAPI 3.1と`/docs`、ハンドラはRustの関数(段階C)または埋め込みPython(段階B、`--features python`、呼び出しごとに正しくGILを取る)。`scripts/test-rust-server-wsl.sh` で本物のFastAPI(uvicorn)と比較し、**段階C 941件・段階B 394件で差異0**、OpenAPIも一致。負荷試験(16並列×2000件、`scripts/load-test-rust-wsl.sh`)は失敗0(段階C `/fib/30` 約2,300 req/s、段階B `/py/hello` 約2,400 req/s。Python処理はGILで直列化される)。`compat.yml` が段階Cのサーバーを毎回FastAPIと比較する。
> - 既知の差(Rustサーバー): 要求の`#`以降はhyperが取り除く(実クライアントは送らない)。GETのみ(他は405)。パスパラメータは区間まるごとの`{name}`/`{name:type}`のみ抽出。JSONの書き出しは`serde_json`(RS-JSON方針の`rust_json::fast`への置換は未実施)。
> - 完了(2026-11-16): Windows 32bit(win32)wheelは配布済み(v0.2.1で確認)、Rustサーバーの単体バイナリを6環境(Linux x64/ARM64・Windows x64/ARM64・macOS Intel/Apple silicon)でビルドしReleaseへ添付(v0.2.2〜、起動して`/fib/10`=55を確認する試験つき)。未了 / Not yet: PyPI公開(アカウントが要る)、実機(Android/iOS/BSD/TV)。 / Done (2026-11-16): win32 wheels are shipped (confirmed in v0.2.1); the standalone Rust server binary is built for 6 platforms (Linux x64/ARM64, Windows x64/ARM64, macOS Intel/Apple silicon) and attached to the Release from v0.2.2, with a start-and-request smoke test (`/fib/10` = 55). Not yet: deciding PyPI publication (needs an account), real devices (Android/iOS/BSD/TV).

Mojo Support for Python's FastAPI. (FastAPI の Python + Rust + C++ 互換、Mojo 対応)

設計: [DESIGN.md](DESIGN.md)。本家: https://fastapi.tiangolo.com/ ・ https://github.com/fastapi/fastapi

## 現状(2026-11-14、段階Aの最小実装)
- `mfastapi.collect_ir(app)`: FastAPIアプリのルート表(IR、JSON直列化可)を取り出す。`include_router`、`Depends`の再帰、path/query/header/cookie/bodyに対応。
- `mfastapi.ir_to_openapi_skeleton(ir)` と `mfastapi.diff_openapi(openapi, skeleton)`: 本家 `/openapi.json` との互換差分テスト(パス・メソッド・パラメータ・本文有無)。
- `mojo/mfastapi_native.mojo`: `PythonModuleBuilder`による最小のMojo拡張(`add`、パス照合`path_matches`)。WSL上のMojo 1.3.0.dev2026111316でビルドし、Pythonから呼べることをpytest 2件で確認。
- 未実装: Rustサーバー、段階B/C、スキーマ本体(型)の比較。`mfastapi_native.mojo`の`path_matches`は初期の簡易版でStarlette非互換(互換版は下記のC ABI版)。

## テスト
```bash
python -m pip install fastapi pytest
python -m pytest -q tests
```
確認環境: Python 3.14, FastAPI 0.143.3, Pydantic 2.14.1(3件成功)。
新しいFastAPIは`include_router`を実行時に展開するため`fastapi.routing.iter_route_contexts`を使う(旧版は`app.routes`へフォールバック)。

## Mojo拡張(WSL)
```bash
wsl -d Ubuntu -- bash /mnt/f/MFastAPI/scripts/setup-mojo-wsl.sh   # 初回: uv + Mojo(nightly)導入
wsl -d Ubuntu -- bash /mnt/f/MFastAPI/scripts/build-mojo-wsl.sh   # ビルド + tests/mojo 実行
```
PowerShellから実行する(Git Bashは`/mnt/...`を書き換える。Windows側PATHの括弧で壊れるため、必ずスクリプト経由)。
Mojoの構文はnightlyで変わる(`len(String)`不可→`byte_length()`など)。版は`setup-mojo-wsl.sh`で固定していないため要注意。

## Python 2 / 3 の両方から呼ぶ(C ABI + ctypes)
- `mojo/mfastapi_cabi.mojo`: C ABI(`mfastapi_abi_version` / `mfastapi_add` / `mfastapi_path_matches`)の共有ライブラリ。Python C APIに依存しないため版を問わない(静的リンクで`ldd`上の依存なし)。
- `python/mfastapi/native.py`: ctypesの薄いラッパー(Python 2.7/3.x構文の共通部分のみ)。ライブラリは`MFASTAPI_LIB`または同ディレクトリの`libmfastapi.so`から探す。
- 確認(WSL, Linux x86_64): Python **2.7.18 / 3.9 / 3.12 / 3.14** で `tests/mojo/test_cabi.py` が成功。
```bash
wsl -d Ubuntu -- bash /mnt/f/MFastAPI/scripts/build-py27-wsl.sh   # 検証用のPython 2.7を~/py27へ(任意)
wsl -d Ubuntu -- bash /mnt/f/MFastAPI/scripts/test-cabi-wsl.sh    # ビルド+2.7/3.xで実行
```
**制限(重要)**: 本家FastAPI・Pydantic・Starletteは**Python 3専用**なので、Python 2から使えるのは`native.py`のネイティブ関数(パス照合など)だけ。FastAPIアプリのIR収集(`collect_ir`)はPython 3のみ。
Python 2はEOL(2020)のため検証用に限り、Python 2.7のビルドにはモダンGCC向けに`-std=gnu11`が必要だった。未確認: Windows(`.dll`)・macOS、Mojo拡張(PythonModuleBuilder版)との性能比較。

## 対応OS(ネイティブ層)
Mojoの公式対応はLinux(x86-64/ARM64)・macOS(Apple silicon)で、Windowsは**WSL経由のみ**(ネイティブ対応は予告段階)。Android・iOSの記載は見つからなかった。
そのためネイティブ層は **同じC ABIを2つの実装で提供**する: Mojo版(`mojo/mfastapi_cabi.mojo`)と、どこでも動く移植用C版(`c/mfastapi_cabi.c`、C99、C標準ライブラリのみ)。Pythonからは`native.py`(ctypes)が同じ名前で探して読む。

| OS | 使う実装 | 状態(この環境での実測) |
|---|---|---|
| Linux (x86-64) | Mojo / C | **確認済み**: 両実装でPython 2.7・3.12のテスト成功、2実装のパリティ(ランダム20万件)一致 |
| Windows 10/11 | C(MSVC) | **確認済み**: `scripts/build-windows.ps1`で x64(Python 3.13)・x86(Python 3.14 32bit)のDLLが成功。Mojoは**WSL経由のみ** |
| Android | C(NDK clang) | **ビルドのみ確認**: arm64-v8a / armeabi-v7a / x86_64 / x86 の`libmfastapi.so`を生成(ELFの機種を確認)。実機・エミュレータでの**実行は未確認**。Chaquopy等のPythonランタイムから名前でロードする想定 |
| macOS | Mojo(Apple silicon) / C | **未確認**(macOS機が無い)。`CMakeLists.txt`のC版か`mojo build --emit shared-lib` |
| iOS / iPhone | C(静的ライブラリ) | **未確認**(Xcode無し)。`cmake -DCMAKE_SYSTEM_NAME=iOS`で`mfastapi_static`を作りアプリへリンクする設計。iOSではPythonも組み込み(BeeWare等)が必要でctypesの制約あり |
| FreeBSD等のUnix | C | **未確認**(`CMakeLists.txt`で`.so`を作る想定) |

```bash
wsl -d Ubuntu -- bash /mnt/f/MFastAPI/scripts/test-cross-wsl.sh   # Linux: C版+Mojo版、Python 2.7/3.xとパリティ
.\scriptsuild-windows.ps1                                       # Windows (PowerShell)
bash scripts/build-android.sh                                      # Android NDK (4 ABI、build/android/)
```
注意: ここでの「対応」は**ネイティブ関数(パス照合など)の呼び出し**までで、FastAPI本体(Python 3専用)をAndroid・iOSで動かすことは含まない。

## Starletteとの突き合わせと性能(2026-11-14実測、Starlette 1.8.0)
**調査(Starletteのソース)**: `compile_path()`はテンプレートを正規表現`^...$`にして`re.match`する。`{name}`/`{name:str}`=`[^/]+`、`:path`=`.*`、`:int`=`[0-9]+`、`:float`=`[0-9]+(\.[0-9]+)?`、`:uuid`=8-4-4-4-12の16進(`-`省略可)。パラメータは区間の途中にも置け、`re`と同じ貪欲+バックトラック。不正な`{`はリテラル、未知の型名は`assert`で拒否。`$`は末尾の改行1つ手前でも一致する。

**差分ファジング**(`bench/diff_starlette.py`: ランダムなテンプレート×パス、`starlette.routing.compile_path`を正解とする):
- 最初の実装(区間単位のみ): 比較86,612件中 **13,970件(16%)が不一致**(型指定・区間内パラメータ・波括弧)。
- Starlette互換に作り直し後(C版・Mojo版とも): 25.9万件×3回の乱数で**不一致0**。途中で見つかった差: Starletteは**コードポイント単位**なので、UTF-8の途中でパラメータを区切ると誤一致(`/{y}{x}`と`/٣`)→境界で区切らないよう修正。
- 未対応/差の残る点: パラメータ名の重複(Starletteは`ValueError`)の検出、ホスト用テンプレート(`/`で始まらない)、Starletteの独自コンバータ登録。バックトラック爆発は上限(20万ステップ)で`-2`(Pythonでは`RuntimeError`)にして断念する(Starletteの`re`は上限なし)。

**性能**(`bench/bench_match.py`、Python 3.12・WSL Ubuntu、1回の探索あたりのns。Starlette=コンパイル済み正規表現を順に`match`):

| ルート数 | 当たる位置 | Starlette | ネイティブ1件ずつ | ネイティブ一括(C版) | 一括(Mojo版) |
|---|---|---|---|---|---|
| 10 | 先頭 | 483 | 1700 | 1435 (0.3倍) | 1478 (0.3倍) |
| 50 | 末尾 | 11,957 | 85,363 | 2,975 (4.0倍) | 3,458 (3.4倍) |
| 200 | 末尾/外れ | 46,727 / 44,411 | 343,921 / 332,701 | 6,732 / 5,831 (7.6倍) | 10,264 / 9,399 (4.4倍) |

- **結論(正直に)**: 照合の中身そのものは速い(C版で約22ns/ルート、Starletteは約220ns/ルート=約10倍)。しかし**ctypesの呼び出し固定費が約0.7〜0.9µs**あり、先頭付近で当たる探索や1件ずつの呼び出しでは**Starletteより遅い**(0.3倍)。得をするのは、ルートが多く(≥約30)深く探す場合の**一括呼び出し**(`route_first_match`)で3〜8倍。
- 呼び出し固定費の実測(`bench/bench_overhead.py`、`add(2,3)`): 純Python演算25ns / ctypes 735〜916ns / **MojoのPythonModuleBuilder拡張 141ns**。→ 先頭ヒットでもStarlette(約480ns)に勝つには、一括照合を拡張モジュール版(Python 3専用)で提供するのが筋(未実装)。ctypes版(Python 2/他OS)は大きなルート表向け。
- Mojo版はC版より約1.5〜2倍遅い(構造体経由の読み出し等、最適化は未着手)。測定はこの1台・Linux(WSL)のみ。Windows/Android/macOSの速度は未測定。
- 本物のStarletteは照合の後に`scope`の更新やパラメータ変換(`convert`)も行うため、実アプリでの全体の速度向上はこの表より小さい。

## 拡張モジュール版の一括照合(Python 3専用、2026-11-14)
`mojo/mfastapi_native.mojo`(`PythonModuleBuilder`)に **`RouteTable`** を追加した。ルート登録時にテンプレートを一度だけ解析して(リテラルは連結、パラメータは種別コード化)保持し、`table.first_match(path)`が最初に一致した番号(なければ`-1`)を返す。`mojo/mfastapi_match.mojo`(照合の核)・`mojo/mfastapi_table.mojo`(コンパイル済み表)をC ABI版と共有する。
```python
import mfastapi_native as ext          # build: scripts/build-ext-wsl.sh
t = ext.RouteTable(["/", "/items/{id:int}", "/items/{name}"])
t.first_match("/items/7")   # -> 1
```
`mfastapi.native.make_table(templates)` は拡張があればそれ、無ければctypes版を返す(Python 2でも使える)。

**速さを出すために入れた工夫**(それぞれ実測で確認): ①ctypesをやめて拡張にし呼び出し固定費を0.74〜0.9µs→約0.14µsへ、②Pythonのstrが持つUTF-8バッファをCPythonから**コピーなしで**直接参照(`PyUnicode_AsUTF8AndSize`、1回あたり約80ns短縮)、③テンプレートを事前解析して毎回の再解析をなくす、④先頭のリテラルを関数呼び出し前にその場で比較して足切り、⑤末尾の`{str}`は取り切れたらバックトラックなしで一致。

**正しさ**: `bench/diff_table.py`(ランダムな表×パスをStarletteの正規表現と比較): Python 3.12・3.14で各16万件、**不一致0**。`scripts/test-ext-wsl.sh`で再現できる。

**性能**(`bench/bench_match.py`、Python 3.12・WSL、1回の探索あたりns。Starlette 1.8.0 = コンパイル済み正規表現を順に`match`):

| ルート数・当たる位置 | Starlette | ctypes一括 | **拡張 RouteTable** | 拡張の倍率 |
|---|---|---|---|---|
| 10・先頭 | 470 | 1,462 | **253** | 1.9倍 |
| 10・末尾 | 2,286 | 1,947 | **347** | 6.6倍 |
| 50・末尾 | 12,454 | 3,526 | **664** | 18.8倍 |
| 200・末尾 | 47,768 | 11,716 | **2,079** | 23.0倍 |
| 200・外れ | 47,210 | 8,665 | **1,292** | 36.5倍 |

先頭ヒットでもStarletteに勝つ(前回のctypes版は0.3倍=負け)。残る固定費は約230ns(Pythonのメソッド呼び出し約140ns+その他)で、これ以上はPython側の呼び出し機構の限界に近い。
注意: ①実アプリのStarletteは照合後に`scope`更新・パラメータ変換も行うため全体の改善はこの表より小さい。②この1台・Linux(WSL)・Python 3.12のみの測定。③パラメータ値の取り出しは下記の`match()`で対応。④Mojo拡張のためPython 2・Windowsネイティブ・Android・iOSでは使えない(ctypes/C版にフォールバック)。

### パラメータ値の取り出し `RouteTable.match`(2026-11-14)
`table.match(path)` は一致しなければ `None`、一致すれば `(ルート番号, path_params)` を返す。値はStarletteの`convertor.convert`と同じ型に変換する(`int`→`int`、`float`→`float`、`uuid`→`uuid.UUID`、`str`/`path`→`str`。19桁以上の整数はPythonの`int()`で変換)。
```python
t = ext.RouteTable(["/users/{uid}/posts/{pid:int}"])
t.match("/users/alice/posts/77")   # -> (0, {'uid': 'alice', 'pid': 77})
```
- 登録時に、パラメータ名の重複(Starletteと同じく`ValueError`相当)・17個以上のパラメータ・未知の型名を拒否する。
- 内部: 照合中に各パラメータの[開始,終端]を記録(バックトラックで上書きされ、成功した経路のものだけが残る)。パラメータ名のstrと`int`/`float`/`UUID`は表に保持し、毎回の`import`・文字列生成をなくした(これで`float`が0.7倍→1.6倍に改善)。
- **正しさ**: `bench/diff_table.py`が`match`の結果(値と**型**)をStarletteの`groupdict()`+`convert`と突き合わせ、Python 3.12・3.14で各16万件・不一致0。

**性能**(`bench/bench_params.py`、Python 3.12・WSL、Starlette = `match`+`groupdict`+`convert`、1回あたりns):

| ケース | Starlette | `ext.match` | 倍率 |
|---|---|---|---|
| 静的 `/health` | 811 | 436 | 1.9倍 |
| `/items/{id:int}` | 1,271 | 598 | 2.1倍 |
| `/users/{uid}/posts/{pid:int}` | 1,599 | 837 | 1.9倍 |
| `/files/{p:path}` | 1,132 | 643 | 1.8倍 |
| `/m/{x:float}` | 1,370 | 864 | 1.6倍 |
| `/u/{u:uuid}` | 3,377 | 2,953 | 1.1倍 |

`uuid`はPython標準の`uuid.UUID(...)`の生成自体が遅く(約2.5µs)、そこは両者共通のため差が出ない。値を取り出さない`first_match`(約250ns)に比べ`match`が約180ns遅いのは、結果の辞書とタプルの生成費用。

**(以前の判断: trie化は見送り → 下記のとおり実施した)** 当時の記述:: 200ルート末尾でも約3.5µs(1ルートあたり約17ns、Starletteの約13.6倍速い)で、効くのは数百〜数千ルートのときだけ。実装の複雑さ(順序を保つ候補集合の併合)に見合う利得が測れていないため、未実装。必要になったら`bench_match.py`のNを増やして再評価する。

## trie(先頭リテラルの木)化(2026-11-14)
`RouteTable`は、各ルートの先頭リテラルを**バイト単位のtrie**に載せる。検索は受信パスをたどりながら、経路上のノードに載っているルートだけを**番号の若い順に**試し、最小の番号を採用する(線形版と同じ結果)。ルートが8個以下なら線形のまま(木をたどるほうが遅いため)。途中で見つけた不具合: 後続の試行が失敗してもキャプチャ領域を上書きするため、採用したルートのキャプチャを別領域(`bcaps`)へ退避するようにした。
- **正しさ**: `bench/diff_table.py`を最大150ルートの表まで広げ、trie版・線形版・Starlette(+`first_match_from`)を突き合わせ、Python 3.12・3.14で各16万件・**不一致0**。
- **性能**(`bench/bench_match.py`、1回あたりns、Python 3.12・WSL):

| ルート数・位置 | Starlette | 線形 | **trie** | trieの対Starlette |
|---|---|---|---|---|
| 200・末尾 | 49,555 | 1,953 | **340** | 146倍 |
| 1000・末尾 | 244,514 | 6,660 | **379** | 646倍 |
| 3000・末尾 | 784,885 | 18,425 | **599〜361** | 1,300〜2,190倍 |
| 3000・外れ | 709,614 | 16,376 | **208〜340** | 2,700〜3,400倍 |
| 10・先頭 | 480 | 261 | 318 | 1.5倍(線形のほうが速いので8個以下は線形) |

走査コストがルート数に依存しなくなった(約200〜600ns一定)。なお測定の揺れは±数十%ある(同じ構成を繰り返すと値が変わる)。

## StarletteのRouter / FastAPIへの差し込み `mfastapi.accelerate(app)`(2026-11-14)
```python
import mfastapi
app = FastAPI(...)   # または Starlette(...) / Router(...)
mfastapi.accelerate(app)   # ルート登録の後に呼ぶ。要 Python 3 + mfastapi_native
```
**設計(安全第一)**: ルート探索の「候補の絞り込み」だけを置き換え、判定は本家に任せる。①パスで`RouteTable.first_match_from`を引き、パスが合う候補ルートを番号順に得る。②候補に本家の`route.matches(scope)`を呼ぶ(405のPARTIAL・`child_scope`・`path_params`の変換は本家のまま)。③最初のFULLを本家と同じ手順で処理(Starlette素の`Router.app`の分岐/FastAPIの`APIRouter.app`の分岐を再現)。④**FULLが無い場合は元の`Router.app`にそのまま委ねる**(405・redirect_slashes・FastAPIの低優先度・404の例外経路の挙動は一切変えない)。例外: プレーンなStarletteで「候補が1つも無く、redirect_slashesの候補も無い」場合だけ、全ルート走査を省いて直接`default`(404)へ進む。テンプレートを持たないルート(Host・独自BaseRoute・FastAPIの`include_router`由来など)は「全パス一致の候補」として表に入れ、本家の`matches`で最終判定する(絞り込みが緩いだけで結果は変わらない)。ルート数が8未満なら何もしない。ルート数や末尾が変わると表を作り直す(途中への挿入・並べ替えで数と末尾が同じ場合は`router._mfastapi_state`を消す必要がある)。

**等価性テスト**(`tests/accel/test_router_equiv.py`): Starlette・FastAPI(`include_router`のprefix・Mount・型付きパラメータ・405・末尾スラッシュ・redirect_slashes・日本語パス・多数のメソッド)の各315リクエストで、`accelerate`の有無を比較(ステータス・`Location`・`Allow`・本文)。Python 3.12・3.14で**不一致0**。FastAPIで実際に加速されていること、`accelerate`後に追加したルートが効くことも確認。

**性能**(`bench/bench_router.py`、何もしないASGIエンドポイントでRouter呼び出し全体を測定、ns/回、Python 3.12・WSL、Starlette 1.8.0):

| ルート数 | 当たる位置 | Starlette | accelerate | 倍率 |
|---|---|---|---|---|
| 10 | 先頭 | 3,843 | 4,954 | **0.8倍(遅い)** |
| 10 | 末尾 | 10,599 | 5,212 | 2.0倍 |
| 50 | 末尾 | 40,178 | 6,127 | 6.6倍 |
| 200 | 末尾 | 141,799 | 7,365 | 19.3倍 |
| 1000 | 末尾 | 662,984 | 8,508 | 77.9倍 |
| 200 | 404(外れ) | 261,517 | 8,531 | 30.7倍 |
| 1000 | 404(外れ) | 1,322,578 | 9,555 | 138倍 |

**正直な注意**: ①先頭のルートに当たる場合は本家が正規表現1回で済むため、一定の追加費用(約1µs)の分だけ**遅くなる**(10ルートで0.8倍)。損益分岐はおよそ3番目のルート。ルートが均等に使われるなら10ルートでも平均で得になるが、特定の先頭ルートに偏る場合は損。②速度はStarletteの素の`Router`で測った(FastAPIの実測は下の節)。③FastAPIの内部ヘルパ`fastapi.routing._route_selected`(非公開)に依存する(FastAPI 0.143.3で確認、無ければ本家へ委ねる)。④Starlette 1.8.0 / FastAPI 0.143.3 のみで検証。⑤`Router(middleware=...)`を使っている場合は警告して加速しない。⑥WebSocketは専用の等価性テストを追加した(下記)。

### 未完了だった項目の仕上げ(2026-11-14)
- **C版コンパイル済みルート表**(`c/mfastapi_cabi.c`の`mfastapi_table_*`、ctypesラッパー`mfastapi.native.CTable`): Mojo拡張が使えない環境(Windows等)でも`accelerate`が動くよう、`mfastapi_native`が無ければ自動でこれに切り替える。テンプレートを1回だけ保持し、**先頭リテラルのバイトtrie**で候補を絞ってからStarlette互換の照合をする(Mojo版と同じ考え方。同じ節のルートは番号順に連結し、最小番号の一致を返す)。`match`(path_params取得)は未提供。当初は線形走査だった(3000ルートの外れで7.4µs)が、trie化で**ルート数によらずほぼ一定**になった(`bench/bench_ctable.py`、ns/回、Python 3.12・WSL、ctypes呼び出しの約1.4µsを含む): 1000ルート末尾 1.6µs(Starletteの166倍)、3000ルート末尾 1.7µs(475倍)、3000ルート外れ 1.4µs(547倍)。先頭ルートに当たる場合は0.7〜0.8倍(ctypesの固定費のため遅い)。ライブラリの読み込みに失敗する例外はすべて本家へ委ねる。
- **C版の差分検証**(`bench/diff_ctable.py`): `first_match`と`first_match_from`をStarletteの`compile_path`と突き合わせ、**16万件×(Linux gcc・Python 3.12/3.14)で不一致0**、**4万件×(Windows MSVC x64・Python 3.13)で不一致0**。
- **CPython C拡張 `mfastapi_cext`**(`c/mfastapi_pyext.c`、`setup_cext.py`、Python 3専用): 同じC版ルート表をctypesを介さず呼ぶ。`accelerate`の優先順位は Mojo拡張 → `mfastapi_cext` → ctypes版。ctypesの固定費(約1.4µs)が消え、**先頭ルートに当たる場合でもStarletteの約4倍速い**(0.8倍→3.7〜4.5倍)。`first_match`は10〜3000ルートで0.18〜0.40µs(ルート数によらずほぼ一定、3000ルート外れでStarletteの約4,000倍(Linux)〜8,000倍(Windows))。Starlette差分(Linux Python 3.12/3.14で各16万件・Windows MSVC x64 Python 3.13で4万件)・等価性315件×2・WebSocket 7パスとも不一致0。Pythonのバージョンごとにビルドが必要(`scripts/test-cext-wsl.sh`・`scripts/build-windows.ps1`)。未配布(ホイール化・PyPI公開は未着手)。**ルーター全体(`bench_router.py`・`bench_router_fastapi.py`)をcext経由で測り直した結果(Linux・Python 3.12)は、Mojo拡張経由とほぼ同じ**: Starlette素のRouterは10ルート先頭0.8倍・末尾2.1倍、200ルート末尾19倍、1000ルート末尾88倍・外れ155倍。FastAPIは1000ルート末尾3.7倍、200ルート末尾1.6倍、先頭・外れは約1.0倍。つまりルーター全体では、照合そのもの(0.2〜0.4µs)ではなくPython側の差し込み処理(約4µs)が残りを占めており、表の高速化(Mojo→cext→trie)はこれ以上は全体に効かない。10ルート先頭が0.8倍(約0.8µs遅い)なのは、この差し込み処理の固定費のため。削るならPython側の処理(コルーチンの二重化・scopeの辞書参照)が対象で、未着手。
- **パッケージ化**(`pyproject.toml`・`setup.py`・`MANIFEST.in`、名前`mfastapi` 0.1.0): `pip install .`で、C拡張`mfastapi_cext`は**任意(optional)**としてビルドされる(コンパイラが無ければ拡張なしで入り、ctypes版/本家のまま動く)。WSLで`uv build --wheel`→新しいvenvへ導入→`accelerate`がcext経由で動き200を返すことを確認(`scripts/test-wheel-wsl.sh`)。WSLで作れるのは`linux_x86_64`の環境依存ホイール、Windowsでは`cp313-win_amd64`ホイールを実際にビルド→新しいvenvへ導入→cext経由で等価性テスト(315件×2・WebSocket)不一致0を確認した。配布用の`manylinux`・macOS(Intel/Apple silicon)・Windows・Linux ARMのホイールは、cibuildwheelのGitHub Actions(`.github/workflows/wheels.yml`、各ホイールで等価性テストを実行、成果物はアーティファクト保存)を用意したが、**このPCでは実行できないため、GitHub Actionsで手動実行した(2026-11-14、公開リポジトリなので無料枠)。結果: Windows・macOS(Apple silicon)・Linux ARMの各ジョブは成功、Linux x86_64のmanylinuxホイールも全バージョンで等価性テスト不一致0。失敗は2件で、いずれもCI側の設定が原因: ①`manylinux i686`のコンテナはネットワークが無く依存取得に失敗(→32bit Linuxは対象外にした) ②`macos-13`ランナーが存在せず待機のまま(→`macos-15-intel`へ変更)。この2点を直して再実行した結果、全6ジョブ(Linux x86_64・Linux ARM・Windows・macOS Intel・macOS Apple silicon・sdist)が成功し、各ホイールで等価性テストが通った**。**GitHub Release `v0.1.0`**(https://github.com/aon-co-jp/MFastAPI/releases/tag/v0.1.0)に、Python 3.9〜3.13のホイール30本(manylinux x86_64/aarch64、macOS Intel/Apple silicon、Windows 64/32bit)とsdistを自動添付した(タグ`v*`のpushでCIが作る)。cibuildwheelをv4.3.5へ上げ、**Python 3.14(・3.14t)のホイールも作成済み**(Release `v0.1.1`、計55ファイル。`v0.1.0`は3.9〜3.13のみ)。**PyPIへの公開は未着手**(アカウントと信頼できる発行元の設定が必要なため、ワークフローにも含めていない)。
- **Starlette/FastAPIのバージョン**: 等価性テスト(315件×2・WebSocket 7パス)を cext経由で Starlette 0.46.2+FastAPI 0.115.14 / 0.52.1+0.128.8 / 最新(1.10.0+0.162.7)の3組で実行し、すべて不一致0(`scripts/test-versions-wsl.sh`)。古いFastAPIでは`include_router`がワイルドカード候補を作らないため、テスト内の`plain is False`の固定は外した。
- **Python側の差し込み処理の削減**: 属性参照・定数の事前束縛などの微最適化を入れたが、測定上の改善はほぼ無かった(10ルート先頭 4.66µs→4.63µs、0.8倍のまま)。残る約0.8µsは表の呼び出しと本家と二重になる最小限の手順で、これ以上は構造を変えない限り削れない。
- **等価性テスト**はC版でも合格: Linux(Python 3.12/3.14)・Windows(MSVC x64・Python 3.13)とも、Starlette・FastAPI 各315リクエストで不一致0。
- **WebSocket**: 既存/未登録パス・パスパラメータ付き・余計な末尾など7パスで、Starlette・FastAPIともに`accelerate`の有無で接続結果が一致(Mojo版・C版の両方)。実際に接続できる経路が含まれていることも確認。
- **FastAPIの速度**(`bench/bench_router_fastapi.py`、何もしない軽いルートN本、ASGI呼び出し全体、Python 3.12・WSL・C版): 1リクエストが約0.42〜0.5ms(FastAPIの依存解決・応答生成が支配的)なので、効果は**ルートが多く、かつ末尾に近いものに当たる場合に限られる**。1000ルート末尾で3.5倍(1.74ms→0.50ms)、200ルート末尾で1.5倍。それ以外(先頭・50ルート以下)は1.0倍前後で差なし。**404(外れ)は加速されない**(1000ルートで2.25ms→2.25ms): FastAPIは`include_router`等で全パス一致の候補が混じるため近道を使わず本家に委ねる設計のため。
- 限界: Windowsで検証したのはC版(MSVC x64、Python 3.13)のみ。**ネイティブWindowsのMojo拡張・macOS・Android・iOSは、実行環境が無い/Mojoが未対応のため未検証**(macOSはMojo公式対応だがこのPCに実機なし)。

### 実機に近い環境での検証(GitHub Actions、2026-11-14)
- **Mojo拡張**(`.github/workflows/mojo.yml`): Linux x86_64・Linux ARM64・**macOS Apple silicon**の3環境でMojo(nightly)から`mfastapi_native`をビルドし、Starlette差分4万件・Starlette/FastAPI各315件の等価性・WebSocket 7パスがすべて不一致0(`accelerate`が`mfastapi_native`を使うことも確認)。これでMojo拡張はmacOSでも確認済み。
- **macOS・Windows・Linux ARMのC拡張ホイール**: `wheels.yml`の各ジョブで等価性テストが通っている(上記)。
- **iOS・Android**(`.github/workflows/mobile.yml`、手動実行): cibuildwheelでiOSシミュレーター(`cp313-ios_arm64_iphonesimulator`)・Androidエミュレーター(`cp313-android_x86_64`)向けのホイールを作り、実際にシミュレーター/エミュレーター上で`python -m mfastapi_cext_selftest`(C拡張の依存なしセルフテスト)が合格した。**確認したのはC拡張の照合(iOS・Androidで`fastapi`が入らないためセルフテスト限定)であり、StarletteやFastAPIとの等価性テストは未実施。実機(実際のiPhone・Android端末)・arm64のAndroidエミュレーター・Python 3.13以外は未確認**。モバイルのホイールはReleaseには添付していない。
- **ネイティブWindowsのMojo拡張**: Mojoが公式にWindowsネイティブ対応していない(WSL経由のみ)ため、引き続き未対応(C拡張/ctypes版を使う)。
- **fastapi非依存化とモバイルでの等価性**(2026-11-14): `import mfastapi`と`accelerate`はStarletteだけで動くようにした(IR/OpenAPI関連は使う時に遅延importし、`fastapi`はオプション依存`mfastapi[fastapi]`)。等価性テストはインストールされるモジュール`mfastapi_equiv_selftest`(`python -m mfastapi_equiv_selftest`)にし、fastapiが無ければStarletteのみ検証する。これにより**iOSシミュレーター・Androidエミュレーター上でも、Starletteの等価性315件+WebSocket 7パスが不一致0**で合格した(`mobile.yml`、`accelerate`がC拡張経由で動作)。FastAPI側の等価性はモバイルでは未実施(pydantic-coreのモバイル用ホイールが無いため)。

## LinuxのWebサーバーで使う(Mojoあり / C拡張 / 拡張なし)(2026-11-14)
**結論: Linuxなら動く。Mojoが無くても、mfastapiだけでも動く。** 実際のWebサーバー(uvicorn、FastAPI・ルート200本)で3つのモードを起動し、同じリクエストを送って確認した(`scripts/test-server-wsl.sh`、`examples/server_app.py`、WSL Ubuntu・Python 3.12):

| モード | 使われる実装 | 準備 |
|---|---|---|
| ① Mojo | Mojo拡張(`mfastapi_native`) | MojoでビルドしたLinux用の`.so`をPYTHONPATHに置く(Linux x86_64/ARM64はMojo公式対応) |
| ② C拡張 | `mfastapi_cext` | `pip install mfastapi`(ホイールにC拡張が入る。Mojo不要) |
| ③ 拡張なし | 本家のルート探索のまま | `PYTHONPATH`に`python/`だけ。コンパイラもMojoも不要(`accelerate`は警告を出して何もしない) |

- `mfastapi.backend()`で、いま使われている実装(`"mojo"` / `"cext"` / `"ctypes"` / `"none"`)を確認できる。環境変数`MFASTAPI_DISABLE=1`で、再デプロイなしに`accelerate`を無効化できる(運用上の安全弁)。
- **結果**: 3モードとも起動でき、8本のリクエスト(各ルート・パスパラメータ・`{p:path}`・404・末尾スラッシュ・`/__backend`)の**ステータスと本文がモード間で完全に同一**だった。
- **速度(uvicornの実HTTP、1リクエストあたり)**: 200ルートで、末尾ルートは拡張なし1,345µs → Mojo 1,088µs・C拡張 1,086µs(約1.24倍)。先頭ルートと404は約1,030〜1,150µsで差なし。HTTPサーバーとFastAPIの処理が大半で、ルート探索の短縮は目立ちにくい。**数十〜数千ルートで、末尾寄りのルートが多いアプリほど効く**(前節のベンチ参照)。MojoとC拡張に実用上の差は無い。
- 注意: 測定は1台のWSL(ループバック接続)のみ。マルチワーカー・本番のnginx/リバースプロキシ配下・負荷下での測定は未実施。

## 段階A+: Mojoで書いた関数をFastAPIのエンドポイントにする `mojo_endpoint`(2026-11-14)
FastAPIの本体(引数の取り出し・Pydantic検証・`Depends`・OpenAPI・`/docs`)はそのまま、**エンドポイントの中身(計算)だけをMojoで書く**。

```python
import mfastapi_handlers as h          # mojo/mfastapi_handlers.mojo をビルドした拡張
from mfastapi import mojo_endpoint
app.add_api_route("/fib/{n}", mojo_endpoint(h.fib, {"n": int}, returns=int), methods=["GET"])
app.add_api_route("/primes", mojo_endpoint(h.count_primes, {"n": (int, 1000)}, returns=int))   # クエリn(既定1000)
```
Mojo拡張の関数にはシグネチャが無いため、`params`で宣言した引数名・型・既定値を持つ薄いPython関数でくるむ(`inspect.Signature`を設定)。Mojo側の`raise`はHTTP 400(`error_status`で変更可)になる。サンプルのMojo関数(`mojo/mfastapi_handlers.mojo`): `fib`・`count_primes`(ふるい)・`word_count`・`mean`。ビルドは`scripts/build-handlers-wsl.sh`、検証は`scripts/test-handlers-wsl.sh`。

**検証**(`tests/mojo_endpoint/test_endpoint.py`、Python 3.12・3.14): 同じ4エンドポイントを純Pythonで書いたものと比較。正常系・型エラー(422、本文もFastAPI生成)・欠落・不正本文を含む14ケースで**ステータスと本文が一致(不一致0)**、OpenAPIの`parameters`・`requestBody`・応答スキーマ・`operationId`も一致、Mojoの`raise`がHTTP 400になることを確認。

**速度**(`bench/bench_handlers.py`、ASGIを直接呼ぶ1リクエストあたり、Python 3.12・WSL、FastAPIの処理込み):

| エンドポイント | 純Python | Mojo | 倍率 |
|---|---|---|---|
| fib(10) | 753µs | 737µs | 1.0倍 |
| fib(25) | 19.5ms | 1.2ms | 16倍 |
| fib(30) | 206ms | 5.2ms | 40倍 |
| 素数の数(n=1,000) | 823µs | 759µs | 1.1倍 |
| 素数の数(n=100万) | 92ms | 12ms | 7.7倍 |
| 素数の数(n=1000万) | 1.18s | 156ms | 7.6倍 |
| 単語数(短文) | 749µs | 753µs | 1.0倍 |

**正直な注意**: ①**計算が軽いエンドポイントでは効果が無い**(約750µsはFastAPIの固定費で、Mojoにしても変わらない)。効くのは1リクエストの計算が数ms以上のCPU負荷型。②Mojo関数の実行中は**GILを保持**するため、同じプロセスの他のPythonスレッドは止まる(長い計算は別ワーカー/プロセスで)。③`mean`のようにPythonのlistを渡す関数は、変換費用で得にならない場合がある(未測定)。④サンプル関数は動作確認用で、実アプリのロジックはこの形で自分で書く。⑤検証はLinux(WSL)のみ。

## 段階B・C: Mojo(flare)がHTTPとルーティングを担当する(2026-11-14)
Mojo製のネットワークスタック[flare](https://github.com/ehsanmok/flare)(MIT、HTTP/1.1・HTTP/2・HTTP/3・WebSocket・TLS、型付き抽出器、Router)を使い、**FastAPI/uvicornを通さずMojoだけでHTTPを受ける**サーバーを作った。flareはMojo 1.2.0(pixi経由、`scripts/setup-flare-wsl.sh`)で動かした(段階Aのuv版nightly 1.3.0.devとは別環境)。

### 段階C: 純Mojoの型付きハンドラ(`mojo/server_c.mojo`、`scripts/build-server-c-wsl.sh`・`test-server-c-wsl.sh`)
`GET /fib/:n`・`/primes?n=`・`/words?text=`・`/health`。Pythonはリクエスト経路に一切入らない。`PathInt`・`OptionalQueryInt`・`QueryStr`の型付き抽出器で、不正な値は**ハンドラ実行前に400**になる。
- **応答の比較**(FastAPI+Mojo関数版と同じ要求): 正常系(fib・primes・words・health)の**本文とステータスは一致**。404も一致(本文の形式は異なる)。
- **差異(正直に)**: ①型が合わない入力は、FastAPIが**422**+詳細JSON、純Mojo版は**400**+`Bad Request`(エラー形式の互換は未実装)。②`QueryStr`はパーセントデコードされないため、`urldecode`を自前で呼んでいる(最初は`%20`のまま数えて単語数が1になる不具合があり、修正した)。③OpenAPI・`/docs`はこのサーバーでは未提供(flareにOpenAPI出力があるが未接続)。
- **速度**(逐次・keep-alive、1リクエストあたり、Python製クライアント込みのループバック): `/health` 純Mojo 320µs vs FastAPI 752µs、`/fib/10` 317µs vs 1,560µs(FastAPIはMojo関数でも固定費が大きい)。`/fib/30`・`/primes?n=100万`は計算が支配的で 5.2ms vs 5.8ms・9.0ms vs 12.5ms。クライアント側のオーバーヘッドが大きく、**サーバー単体の限界性能は未測定**(負荷ツール未使用)。

### 段階B: Mojoが受け口、利用者のハンドラはPython(`mojo/server_b.mojo`、`examples/pyhandlers.py`、`scripts/build-server-b-wsl.sh`)
Mojo(flare)がHTTP・ルーティング・型付き抽出を担当し、`/py/...`のルートだけ、同じプロセスに埋め込んだPythonの関数(`pyhandlers.greet`・`fib`・`stats`)を境界越えで呼ぶ。同じサーバーにMojoだけのルート(`/fib/:n`)も共存する。
- 動作確認: `/py/hello/Taro`・`/py/fib/20`・`/py/stats?csv=1,2,3.5`・`/fib/20`・範囲外400・404がすべて期待どおり。
- **境界越えの費用**(1リクエスト): `/health` 323µs に対し `/py/hello/Taro` 350µs(Pythonを呼ぶ追加費用は約30µs)。計算の中身はPythonのままなので、`fib(30)`は純Mojo版 5.2ms に対しPython版 243ms(約47倍)。つまり**重い所はMojoのルートに置き、利用者の柔軟なロジックだけPythonに任せる**使い分けになる。
- **制限**: Pythonは1プロセス1インタープリタ(GIL)のため、`num_workers=1`に固定している(複数ワーカーでの安全なGIL処理は未検証)。`MOJO_PYTHON_LIBRARY`で`libpython`を指定する必要がある。FastAPIの`Depends`・Pydantic検証・ミドルウェアは使えない(Python側の関数は文字列を受けて文字列を返すだけ)。
- 検証はLinux(WSL)のみ。flareは外部の若いプロジェクト(2026-11時点)で、APIが変わる可能性がある。

### 段階B・CでもOpenAPIと422をFastAPIにそろえる(`mojo/mfastapi_web.mojo`、2026-11-14)
段階B・CのMojoサーバーも、FastAPI(+Mojo関数版)と同じ形式で次を返す。
- **422**: `{"detail":[{"type":"int_parsing","loc":["path","n"],"msg":"Input should be a valid integer, unable to parse string as an integer","input":"abc"}]}`、必須欠落は `type:"missing"`(`input:null`)。
- **`/openapi.json`**(OpenAPI 3.1、parameters・operationId・422・`HTTPValidationError`/`ValidationError`)と **`/docs`**(Swagger UI)。
- 実測(WSL、`scripts/test-server-compat-wsl.sh`=段階C、`scripts/test-server-b-compat-wsl.sh`=段階B): 応答・422本文は C 17件 / B 12件、OpenAPIは C 3経路 / B 4経路で、FastAPIとの差異 0。
- int解析はPydanticのlaxモードに合わせた("5.0"・" 5 "・"1_000"を受理、"5."・"1e1"・"1__0"は422。FastAPIと25件で一致)。既知の差: 18桁超(64bit)、400の本文は `{"detail":"..."}`、404本文の形式は別、Swagger UIはCDN読み込み、段階Bの200スキーマは型付けなし、段階BはGILのため `num_workers=1`。

### 互換性の確認方法と範囲(2026-11-14)
- 基準は**本物のuvicorn上のFastAPI**(TestClientはパスの`%25`を二重に復号し`.`を正規化するため基準にしない。これはファジングで判明)。Mojoサーバーと同じ要求を送り、ステータスと本文(JSON)を比較する。
- ファジング(seed固定、`tests/mojo_endpoint/test_mojo_server_compat.py`): 整数パラメータに約400種の文字列(空白・`+-`・`_`・`.`・`e`・非ASCII・`%`エンコード等)、クエリのint/str、重複キー(最後が勝つ)、`?n`(値なし)、`+`と`%2B`。**段階C 942件・段階B 394件で差異0**、OpenAPIの各経路・`HTTPValidationError`/`ValidationError`も一致。
- ファジングで見つけて直した差: 重複クエリキー(最初→最後)、`#`はクエリ内の文字として扱う。
- 保証の範囲: 上記の機能(パス/クエリのint・str、422、OpenAPI)。**範囲外**: Depends・Pydanticモデルの本文・ミドルウェア・str以外の型(段階B・Cでは未対応)、18桁超の整数(64bit)、404本文の形式。ルーティング加速層(`accelerate`)は別途 `mfastapi_equiv_selftest` でStarletteと同値を確認している。
- 段階Bの複数ワーカー: 各Python呼び出しで `PyGILState_Ensure` によりGILを取る(以前は `num_workers>1` でsegfault)。負荷試験(`scripts/load-test-wsl.sh`、16並列×2000件): workers=1/2/4とも失敗0・本文一致。ただしGILのためPython処理のスループットはworkersを増やしても伸びない(`/py/hello` 約2,300 req/s、`/py/fib/20` 約460 req/s、段階C `/fib/30` 約400 req/s)。

## API 方針: REST 全廃・gRPC / Connect へ (2026-11-04)

### 日本語
- **REST は全廃する。新しい API は REST で作らない。** ブラウザや `curl` から呼ぶ API は **Connect の単純呼び出し**(`POST /<パッケージ>.<Service>/<Method>`、`content-type: application/json`、本文は JSON)、サービス間は **gRPC / gRPC-web**(RPoem の管理API `open_runo.admin.v1.Admin` と同じ流儀)。ストリーミングは、必要になってから。
- **既存の REST は、触るときに順に移す。** 移したら旧 REST の経路は消す。移行が終わるまで残るが、**新規には追加しない**。
- **JSON は RS-JSON に統一**する(読み込みは `from_str_secure` / `parse_secure` でサイズ・深さ・重複キーの上限つき、書き出しは `rust_json::fast`)。ブラウザの WASM に `prost` は入れない(サイズが増えるため)。
- **例外(自分たちの API ではなく、相手の規格が決めているもの)**: ACME、OAuth/OIDC のコールバック、第三者(GitHub など)からの Webhook、ヘルスチェック、静的ファイル配信、OTLP など。これらは規格どおりの HTTP を受ける。
- 参考実装: `RPoem`(`open_runo.admin.v1.Admin`)、`open-easy-web`(`server/src/connect.rs`、`proto/open_easy_web/admin/v1/admin.proto`)。
- ※ このリポジトリのコードが既に方針どおりかどうかは、各リポジトリの現状による。この節は方針の記録であり、コードの変更を意味しない。

### English
- **REST is being abolished. New APIs are not built as REST.** APIs called from browsers or `curl` use **Connect unary calls** (`POST /<package>.<Service>/<Method>`, `content-type: application/json`, JSON body); service-to-service calls use **gRPC / gRPC-web** (the same style as RPoem's admin API `open_runo.admin.v1.Admin`). Streaming only when it is actually needed.
- **Existing REST endpoints are migrated one by one, when they are touched.** The old REST route is removed once migrated. Until migration is finished they remain, but **no new ones are added**.
- **JSON is unified on RS-JSON**: reading with `from_str_secure` / `parse_secure` (size, depth and duplicate-key limits), writing with `rust_json::fast`. `prost` is not put into browser WASM (it would grow the binary).
- **Exceptions (shapes dictated by someone else's standard, not our own API)**: ACME, OAuth/OIDC callbacks, webhooks from third parties (e.g. GitHub), health checks, static file serving, OTLP, and so on. These accept plain HTTP as the standard requires.
- Reference implementations: `RPoem` (`open_runo.admin.v1.Admin`) and `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Note: whether the code in a given repository already follows this policy depends on that repository. This section records the policy; it does not mean the code has been changed.

### Deutsch (KI-Übersetzung, redaktionell überarbeitet; noch nicht von Muttersprachlern geprüft)
- **REST wird vollständig abgeschafft; neue APIs entstehen nicht mehr als REST.** APIs, die aus dem Browser oder per `curl` aufgerufen werden, verwenden **unäre Connect-Aufrufe** (`POST /<Paket>.<Service>/<Methode>`, `content-type: application/json`, JSON-Body). Zwischen Diensten kommen **gRPC bzw. gRPC-web** zum Einsatz, im selben Stil wie die Admin-API von RPoem (`open_runo.admin.v1.Admin`). Streaming wird erst ergänzt, wenn es tatsächlich gebraucht wird.
- **Bestehende REST-Endpunkte werden schrittweise umgestellt, und zwar immer dann, wenn man ohnehin an ihnen arbeitet.** Sobald ein Endpunkt umgestellt ist, wird die alte REST-Route entfernt. Bis zum Abschluss der Migration bleiben die übrigen bestehen, es werden aber **keine neuen mehr hinzugefügt**.
- **JSON wird einheitlich über RS-JSON verarbeitet**: Gelesen wird mit `from_str_secure` bzw. `parse_secure` (mit Obergrenzen für Größe, Verschachtelungstiefe und doppelte Schlüssel), geschrieben mit `rust_json::fast`. `prost` wird nicht in das Browser-WASM aufgenommen, da es die Datei vergrößern würde.
- **Ausnahmen (Schnittstellen, deren Form ein fremder Standard vorgibt und die nicht zu unserer eigenen API gehören)**: ACME, OAuth/OIDC-Callbacks, Webhooks von Drittanbietern (z. B. GitHub), Health-Checks, die Auslieferung statischer Dateien, OTLP und Ähnliches. Diese nehmen, wie vom jeweiligen Standard verlangt, gewöhnliches HTTP entgegen.
- Referenzimplementierungen: `RPoem` (`open_runo.admin.v1.Admin`) und `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Hinweis: Ob der Code eines einzelnen Repositorys diese Richtlinie bereits erfüllt, hängt vom jeweiligen Repository ab. Dieser Abschnitt dokumentiert lediglich die Richtlinie; er besagt nicht, dass der Code schon angepasst wurde.

### Русский (перевод выполнен с помощью ИИ и отредактирован; носителями языка пока не проверен)
- **От REST мы полностью отказываемся; новые API больше не создаются в стиле REST.** API, к которым обращаются из браузера или через `curl`, используют **унарные вызовы Connect** (`POST /<пакет>.<Service>/<Method>`, `content-type: application/json`, тело запроса — JSON). Для взаимодействия между сервисами применяются **gRPC и gRPC-web** — в том же стиле, что и административный API RPoem (`open_runo.admin.v1.Admin`). Потоковая передача данных добавляется лишь тогда, когда в ней действительно возникает необходимость.
- **Существующие REST-эндпоинты переводятся поэтапно — по мере того, как с ними приходится работать.** После перевода старый REST-маршрут удаляется. Пока миграция не завершена, оставшиеся эндпоинты сохраняются, но **новые не добавляются**.
- **Работа с JSON унифицирована на основе RS-JSON**: чтение — через `from_str_secure` или `parse_secure` (с ограничениями на размер, глубину вложенности и повторяющиеся ключи), запись — через `rust_json::fast`. `prost` в браузерный WASM не включается, поскольку это увеличило бы размер файла.
- **Исключения (интерфейсы, форму которых определяет чужой стандарт, а не наш собственный API)**: ACME, обратные вызовы OAuth/OIDC, вебхуки сторонних сервисов (например, GitHub), проверки работоспособности, раздача статических файлов, OTLP и тому подобное. Такие интерфейсы принимают обычный HTTP, как того требует соответствующий стандарт.
- Эталонные реализации: `RPoem` (`open_runo.admin.v1.Admin`) и `open-easy-web` (`server/src/connect.rs`, `proto/open_easy_web/admin/v1/admin.proto`).
- Примечание: соответствует ли код конкретного репозитория этой политике, зависит от самого репозитория. Данный раздел лишь фиксирует политику и не означает, что код уже изменён.

## API 方針の補足: 用語の整理と使い分け (2026-11-04)

### 日本語
**1つに統一するものではなく、別の層の話です。** 「REST の代わり」になるのは Connect・gRPC・GraphQL だけで、WASM は画面を作る技術です。

| 用語 | 何か | REST の代わりか |
|---|---|---|
| REST | URL と HTTP メソッドで操作を表す API の作り方 | 全廃する側 |
| **Connect** | `POST /サービス名/メソッド名` に JSON を送る形式。`curl` でもブラウザの `fetch` でも呼べる | はい(**採用**) |
| **gRPC** | 同じ「サービス/メソッド」の考え方を、バイナリ(protobuf)と HTTP/2 で行う形式。速く、ストリーミングも得意 | はい(**サービス同士に採用**) |
| gRPC-web | gRPC のブラウザ向けの変形版 | はい(必要なら) |
| GraphQL | 欲しい項目を問い合わせる形式 | 一部だけ(**問い合わせ(読み取り)専用**) |
| WASM | Rust をブラウザの中で動かす仕組み | いいえ(画面を作る技術) |

- **ブラウザの画面や `curl` から呼ぶ API は Connect**(POST、JSON を RS-JSON で読み書き)。ブラウザの WASM は素の gRPC を話せず、Connect なら `fetch` のままで、`prost` を足さずに済む(画面のサイズも増えない)。
- **サービス同士の通信(サーバー間、重い処理、ストリーミング)は gRPC。** 多くは Connect で足りるので、ストリーミングが必要になった所だけ gRPC に上げる。
- **GraphQL は、まとめて問い合わせたいときの読み取り専用。** 書き込み(操作)には使わない。
- **WASM は画面を作る技術として、そのまま使う。** 画面から呼ぶ API が Connect になっただけ。
- 外部の規格が形を決めるもの(ACME、OAuth のコールバック、第三者の Webhook、ヘルスチェック、静的ファイル、OTLP)は、規格どおりの HTTP のまま。
- **迷ったら Connect。**

### English
**These are not alternatives to unify; they sit at different layers.** Only Connect, gRPC and GraphQL can replace REST; WASM is a way to build the UI.

| Term | What it is | Replaces REST? |
|---|---|---|
| REST | API style that expresses operations through URLs and HTTP methods | The thing being abolished |
| **Connect** | `POST /Service/Method` with a JSON body; callable from `curl` and from a browser `fetch` | Yes (**adopted**) |
| **gRPC** | The same service/method idea over binary (protobuf) and HTTP/2; fast, good at streaming | Yes (**for service-to-service**) |
| gRPC-web | A browser-friendly variant of gRPC | Yes (if needed) |
| GraphQL | A query language: ask for exactly the fields you want | Partly (**read-only queries**) |
| WASM | Runs Rust inside the browser | No (a UI technology) |

- **APIs called from browser screens or `curl` use Connect** (POST, JSON read and written with RS-JSON). Browser WASM cannot speak plain gRPC; with Connect it stays a `fetch`, and `prost` is not needed (the binary does not grow).
- **Service-to-service calls (server to server, heavy work, streaming) use gRPC.** Connect is enough in most places; move a call to gRPC only where streaming is actually needed.
- **GraphQL is read-only**, for queries that gather several pieces of data. It is not used for writes (operations).
- **WASM stays as the way to build the UI.** The API it calls simply became Connect.
- Interfaces whose shape an outside standard dictates (ACME, OAuth callbacks, third-party webhooks, health checks, static files, OTLP) stay plain HTTP.
- **When in doubt, use Connect.**

### Deutsch (KI-Übersetzung, sprachlich überarbeitet; noch nicht von Muttersprachlern geprüft)
**Es geht nicht darum, eine einzige Technik auszuwählen – die Begriffe liegen auf unterschiedlichen Ebenen.** Als Ersatz für REST kommen nur Connect, gRPC und GraphQL in Frage; WASM dagegen ist eine Technik, mit der die Benutzeroberfläche gebaut wird.

| Begriff | Was ist das? | Ersatz für REST? |
|---|---|---|
| REST | Ein API-Stil, bei dem Operationen über URLs und HTTP-Methoden ausgedrückt werden | wird abgeschafft |
| **Connect** | `POST /Dienst/Methode` mit JSON im Body; aufrufbar mit `curl` ebenso wie mit `fetch` im Browser | ja (**gewählt**) |
| **gRPC** | Dasselbe Prinzip aus Dienst und Methode, aber über binäres protobuf und HTTP/2; schnell und gut für Streaming geeignet | ja (**zwischen Diensten**) |
| gRPC-web | Eine für den Browser angepasste Variante von gRPC | ja (bei Bedarf) |
| GraphQL | Abfragesprache, mit der man genau die benötigten Felder anfordert | teilweise (**nur für lesende Abfragen**) |
| WASM | Führt Rust-Code direkt im Browser aus | nein (Technik für die Oberfläche) |

- **APIs, die von Browseroberflächen oder per `curl` aufgerufen werden, verwenden Connect** (POST; JSON wird mit RS-JSON gelesen und geschrieben). WASM im Browser kann kein reines gRPC sprechen. Mit Connect bleibt es bei einem gewöhnlichen `fetch`, und `prost` ist nicht nötig – die WASM-Datei wird dadurch nicht größer.
- **Für die Kommunikation zwischen Diensten (Server zu Server, rechenintensive Aufgaben, Streaming) wird gRPC eingesetzt.** In den meisten Fällen reicht Connect aus; auf gRPC wird nur dort umgestellt, wo Streaming tatsächlich gebraucht wird.
- **GraphQL dient ausschließlich lesenden Abfragen**, wenn mehrere Informationen gebündelt abgefragt werden sollen. Für Schreibzugriffe (Operationen) wird es nicht verwendet.
- **WASM bleibt die Technik für die Oberfläche.** Geändert hat sich lediglich, dass die aufgerufene API nun auf Connect beruht.
- Schnittstellen, deren Form durch einen externen Standard festgelegt ist (ACME, OAuth-Callbacks, Webhooks von Drittanbietern, Health-Checks, statische Dateien, OTLP), bleiben gewöhnliches HTTP.
- **Im Zweifel: Connect.**

### Русский (перевод выполнен с помощью ИИ и литературно отредактирован; носителями языка пока не проверен)
**Это не варианты, из которых нужно выбрать один, а технологии разных уровней.** Заменить REST могут только Connect, gRPC и GraphQL; WASM же — это способ создания пользовательского интерфейса.

| Термин | Что это | Заменяет REST? |
|---|---|---|
| REST | Стиль построения API, при котором операции выражаются через URL и HTTP-методы | отменяется |
| **Connect** | `POST /Сервис/Метод` с телом в формате JSON; вызывается как из `curl`, так и из браузерного `fetch` | да (**выбран**) |
| **gRPC** | Тот же принцип «сервис — метод», но поверх двоичного формата protobuf и HTTP/2; работает быстро и хорошо подходит для потоковой передачи | да (**для связи между сервисами**) |
| gRPC-web | Адаптированный для браузера вариант gRPC | да (при необходимости) |
| GraphQL | Язык запросов: клиент запрашивает ровно те поля, которые ему нужны | частично (**только запросы на чтение**) |
| WASM | Позволяет выполнять код на Rust прямо в браузере | нет (технология для интерфейса) |

- **API, которые вызываются из браузерных интерфейсов или через `curl`, строятся на Connect** (POST; JSON читается и записывается через RS-JSON). Браузерный WASM не умеет работать с «чистым» gRPC. С Connect достаточно обычного `fetch`, а `prost` не требуется — размер WASM-файла не увеличивается.
- **Для обмена между сервисами (сервер — сервер, ресурсоёмкие задачи, потоковая передача) применяется gRPC.** В большинстве случаев достаточно Connect; на gRPC переходят только там, где потоковая передача действительно необходима.
- **GraphQL используется только для чтения**, когда нужно получить сразу несколько связанных данных. Для записи (операций) он не применяется.
- **WASM по-прежнему остаётся технологией для интерфейса.** Изменилось лишь то, что вызываемый им API теперь построен на Connect.
- Интерфейсы, форма которых задана внешним стандартом (ACME, обратные вызовы OAuth, вебхуки сторонних сервисов, проверки работоспособности, статические файлы, OTLP), остаются обычным HTTP.
- **Если есть сомнения — выбирайте Connect.**

## 補足: ストリーミングの対応状況 (2026-11-07)

### 日本語
- **server-streaming は対応済み**(`RPoem` の `open-runo-connect-*`、RPoem main `1570371`・`34cb595`)。Connect は `application/connect+json` のエンベロープ、gRPC / gRPC-web でも受ける。途中エラーは終端フレームで返し、件数・時間の上限、切断時の資源解放、実 TCP での `WatchEvents` 試験を通過。**VPS には未反映**。
- **使いどころ**: 変更の監視、進捗の逐次通知、長い生成の逐次返却など、サーバーから複数回返したい所。1回で済む呼び出しは従来どおり Connect の単純呼び出し。client-streaming / 双方向は、必要になってから。
- 独語・露語は未追記(日英のみ)。

### English
- **Server-streaming is supported** (`open-runo-connect-*` in RPoem, main `1570371` / `34cb595`): `application/connect+json` envelopes, also via gRPC / gRPC-web; mid-stream errors end with a terminal frame; item/time limits, resource release on disconnect, and a real-TCP `WatchEvents` test pass. **Not yet deployed to the VPS.**
- **When to use**: watching changes, progress updates, incremental results. One-shot calls stay plain Connect unary. Client-streaming / bidirectional only when needed.
- German and Russian not yet added.

## 補足: ストリーミングの対応状況 (2026-11-07)

### 日本語
- **server-streaming は対応済み**(`RPoem` の `open-runo-connect-*`、RPoem main `1570371`・`34cb595`)。Connect は `application/connect+json` のエンベロープ、gRPC / gRPC-web でも受ける。途中エラーは終端フレームで返し、件数・時間の上限、切断時の資源解放、実 TCP での `WatchEvents` 試験を通過。**VPS には未反映**。
- **使いどころ**: 変更の監視、進捗の逐次通知、長い生成の逐次返却など、サーバーから複数回返したい所。1回で済む呼び出しは従来どおり Connect の単純呼び出し。client-streaming / 双方向は、必要になってから。
- 独語・露語は未追記(日英のみ)。

### English
- **Server-streaming is supported** (`open-runo-connect-*` in RPoem, main `1570371` / `34cb595`): `application/connect+json` envelopes, also via gRPC / gRPC-web; mid-stream errors end with a terminal frame; item/time limits, resource release on disconnect, and a real-TCP `WatchEvents` test pass. **Not yet deployed to the VPS.**
- **When to use**: watching changes, progress updates, incremental results. One-shot calls stay plain Connect unary. Client-streaming / bidirectional only when needed.
- German and Russian not yet added.

- **Windows ARM64(Snapdragon)ホイール**(2026-11-15、`wheels.yml`に`windows-11-arm`を追加): GitHub ActionsのARM64ランナーでホイールをビルドし、ホイール内でRustバックエンド確認+等価性テストが合格(6環境すべて成功、Release添付はタグ時)。実機のSnapdragon PCは未確認。

## MFastAPI - 多言語の説明 / Multilingual description (13 languages)

公開ページ / Pages: https://easy-web.tokyo/MFastAPI ・ https://easy-web.tokyo/MFastAPI/demo (ダウンロード専用 / download-only)

- **日本語**: FastAPI互換のPython Webフレームワーク。核心部(ルーティングと入力検証)をRustで高速化。
- **English**: A FastAPI-compatible Python web framework whose core (routing and input validation) runs in Rust for speed.
- **中文(简体)**: 与 FastAPI 兼容的 Python Web 框架,核心部分(路由与输入校验)使用 Rust 加速。
- **中文(台灣・繁體)**: 相容 FastAPI 的 Python Web 框架,核心部分(路由與輸入驗證)以 Rust 加速。
- **한국어**: FastAPI와 호환되는 Python 웹 프레임워크로, 핵심(라우팅과 입력 검증)을 Rust로 가속합니다.
- **Italiano**: Un framework web Python compatibile con FastAPI, il cui nucleo (routing e validazione degli input) è accelerato in Rust.
- **Français**: Un framework web Python compatible FastAPI dont le cœur (routage et validation des entrées) est accéléré en Rust.
- **Deutsch**: Ein FastAPI-kompatibles Python-Webframework, dessen Kern (Routing und Eingabeprüfung) in Rust beschleunigt wird.
- **Русский**: Python-веб-фреймворк, совместимый с FastAPI; его ядро (маршрутизация и проверка входных данных) ускорено на Rust.
- **Українська**: Python-вебфреймворк, сумісний із FastAPI; його ядро (маршрутизація та перевірка вхідних даних) прискорено на Rust.
- **فارسی (ایران)**: چارچوب وب پایتونی سازگار با FastAPI که هستهٔ آن (مسیریابی و اعتبارسنجی ورودی) با Rust سرعت گرفته است.
- **العربية**: إطار عمل ويب بلغة Python متوافق مع FastAPI، وقد جرى تسريع نواته (التوجيه والتحقق من المدخلات) باستخدام Rust.
- **עברית (ישראל)**: מסגרת אינטרנט בפייתון התואמת ל-FastAPI, שליבתה (ניתוב ואימות קלט) מואצת ב-Rust.

Download: https://github.com/aon-co-jp/MFastAPI/releases/latest

## Windows インストーラー方針 / Windows installer policy
**日本語**: インストーラーを持つ製品の Windows 版は、すべて `.exe`(Inno Setup/NSIS、必要なら PyInstaller 同梱)で配布する。PowerShell スクリプトは開発用に残すだけで、利用者向けの入口にしない。RMojo は `RMojo-Setup-x64/arm64.exe` を配布済み、2FastAPI は v0.3.1 から `2FastAPI-Setup-x64/arm64.exe`。MFastAPI は pip の wheel 配布でインストーラーを持たない(Windows でも `pip install mfastapi`)。
**English**: Every product that has an installer ships its Windows build as an `.exe` (Inno Setup/NSIS, with PyInstaller bundling where needed); PowerShell scripts stay for development only. RMojo ships `RMojo-Setup-x64/arm64.exe`; 2FastAPI ships `2FastAPI-Setup-x64/arm64.exe` since v0.3.1. MFastAPI has no installer (wheel only: `pip install mfastapi`, also on Windows).

## 2026-11-16 配布の仕上げ / Distribution finish
**日本語**: Windows 32bit wheel は配布済みと確認(古い「未了」を訂正)。Rust サーバー(`rust/mfastapi-server`)の単体バイナリを Linux/Windows/macOS の x64・ARM64 で CI ビルドし、起動して応答を確かめてから Release へ添付する(`.github/workflows/wheels.yml` の `server` ジョブ、v0.2.2〜)。**追記(2026-11-16)**: Rust サーバーが GET/HEAD/POST/PUT/PATCH/DELETE と JSON 本文に対応(`Route.method`/`Route.body`、`Args::json()`)。本文なしは422 `missing`、不正JSONは422 `json_invalid`、パスは合うがメソッド違いは405+`Allow`、本文1MB超は413、OpenAPI にメソッド別・`requestBody` を出力。CIのスモークにPOST/405/422を追加。 **決定(2026-11-16)**: 純Mojo版サーバー(ステージB/C)は廃止しない。通常の配布・機能追加はRust版(`rust/mfastapi-server`)を正とし、純Mojo版は「正しいMojoとして動くか」の確認テストが必要になったときに使う基準実装として残す(Mojo用CIも維持)。 **未了**: PyPI 公開(所有者のアカウントが必要)、Android/iOS/BSD/TV の実機。
**English**: Win32 wheels are confirmed shipped (the stale "not yet" was corrected). The standalone Rust server (`rust/mfastapi-server`) is CI-built for Linux/Windows/macOS on x64 and ARM64, smoke-tested (it starts and answers) and attached to the Release (`server` job in `.github/workflows/wheels.yml`, from v0.2.2). **Update (2026-11-16)**: the Rust server now handles GET/HEAD/POST/PUT/PATCH/DELETE and JSON bodies (`Route.method`/`Route.body`, `Args::json()`). A missing body gives 422 `missing`, invalid JSON 422 `json_invalid`, a path match with the wrong method 405 + `Allow`, bodies over 1 MB 413; OpenAPI lists per-method operations and `requestBody`. The CI smoke test now covers POST/405/422. **Decision (2026-11-16)**: the pure-Mojo server (stages B/C) is NOT retired. The Rust server (`rust/mfastapi-server`) is the primary one for distribution and new features; the pure-Mojo server stays as the reference implementation to use whenever a check as genuine Mojo is needed (the Mojo CI is kept). **Open**: PyPI publication (needs the owner's account), real Android/iOS/BSD/TV devices.

## 2026-11-23 追記: 手元でのテストの動かし方(日本語)
- Rust コアは `cd rust/mfastapi-core && cargo build --release` で作ります(DLL は `target/release/deps/mfastapi_core.dll`)。環境変数 `MFASTAPI_RS_LIB` にそのパスを、`PYTHONPATH` に `python` フォルダーを指定します。
- **落とし穴**: この PC の既定の `python` は32ビット版で、64ビットの DLL を読めません(`WinError 193`)。64ビット版(`C:\Users\noruk\AppData\Local\Python\pythoncore-3.14-64\python.exe`)を使い、`pytest`・`pydantic`・`httpx`・`starlette`・`fastapi` を入れてください。
- 結果: `tests/test_rust_core.py`(スクリプト形式、整数5158件・クエリ15030件で差0)、`tests/compat` と `tests/test_autoupdate.py` の8件が成功しました。`tests/accel` と Mojo 系は未実施です。
