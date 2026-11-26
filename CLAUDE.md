# mfastapi 開発メモ
- 正本の設計は `DESIGN.md`。性能の主張は実測のみ(Milkhaの倍率は推定値)。
- 新規の自前APIはREST全廃方針(Connect/gRPC)に従う。FastAPI互換の入口は互換目的のみ。
- 次の一手: (1) Rust製ASGIサーバー/IR→OpenAPI全生成、(2) Mojo拡張(`PythonModuleBuilder`)の最小例、(3) スキーマ本体の比較、(4) Flareの版・ライセンス確認。

- 2026-11-14: path_matchesはStarlette互換(差分ファジング不一致0)。性能の結論はREADME参照。次の一手: 一括照合(route_first_match)をPythonModuleBuilder拡張で提供(固定費141ns)、Mojo版の最適化、`collect_ir`のルート表からpack_templatesを自動生成してStarletteのRouterへ差し込む。
- 2026-11-14: 拡張版RouteTable実装(先頭ヒット1.9倍〜外れ36倍)。次の一手: path_params(パラメータ値)の取り出し、先頭リテラルでの木(trie)化でN=200以上をさらに高速化、collect_irからStarlette Routerへ差し込み。
- 2026-11-14: RouteTable.match(path_params)実装済み(Starlette比1.1〜2.1倍、不一致0)。trieは見送り(理由はREADME)。次の一手: collect_irのルート表からRouteTableを作りStarletteのRouter照合へ差し込む(スコープ更新・405/redirect_slashesの扱い)。
- 2026-11-14: trie化(3000ルートで外れ約200ns)・accelerate(Starlette/FastAPIのRouterへ差し込み、等価性テスト不一致0)を実装。同日、C版コンパイル済みルート表(Mojoが無い環境用フォールバック、Linux/Windowsで差分・等価性とも不一致0)・WebSocket等価性テスト・FastAPI速度測定(1000ルート末尾3.5倍、404は加速せず)を完了。その後CIで、Mojo拡張をmacOS Apple siliconでも検証、C拡張のiOSシミュレーター・Androidエミュレーターでのセルフテスト合格、3.14ホイール作成まで完了(v0.1.1)。未了: ネイティブWindowsのMojo拡張(公式未対応)、iOS/Androidの実機とStarlette・FastAPIでの等価性、PyPI公開(要アカウント)。

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

## 2026-11-23 追記: RMojo ハンドラ橋渡し(段階1〜2) / RMojo handler bridge
**日本語**: `rust/mfastapi-server` に機能 `rmojo`(`src/rmojobridge.rs`)を追加。`rmojo_handler(source, func, params, returns, max_steps)` が、Mojo風ソースの関数を RMojo(`F:\RMojo`、`call_limited`)で実行する。専用ワーカースレッド(大きなスタック、コンパイルは1回)、ループ上限超過は504、ハンドラのエラーは400、ワーカー不在は503。`tests/rmojo_bridge.rs` で `/fib/20`=6765・文字列・無限ループ504・不正パス422・504後の復帰を確認。既定機能の試験10件も成功。**未了**: 複数ワーカー(今は直列)、IRの `runtime:"rmojo"` 指定、リスト/辞書の値、時間・メモリ上限、Pythonの `mojo_endpoint` 経由、実HTTP越しの負荷測定。
**English**: Added the `rmojo` feature to `rust/mfastapi-server` (`src/rmojobridge.rs`): `rmojo_handler(...)` runs a function of Mojo-style source in RMojo (`call_limited`) on one dedicated big-stack worker (compiled once). Step-limit overrun gives 504, handler errors 400, missing worker 503. `tests/rmojo_bridge.rs` verifies `/fib/20`=6765, strings, infinite loop 504, bad path 422 and recovery after a 504; the 10 default-feature tests still pass. **Open**: multiple workers (serial today), `runtime:"rmojo"` in the IR, list/dict values, time/memory limits, the Python `mojo_endpoint` path, and real-HTTP load measurements.

## 2026-11-25 現況・未了 / Status and open items

### 日本語

現状: FastAPI 互換で、中身に Rust のコアを持つ Python 用の枠組み。最近は CI が姉妹の RMojo を取得するようにして互換試験(compat)を通し、RMojo のメモリ上限を文書に追記した。この PC では Rust コアが未ビルドのため、確認は純 Python の範囲にとどまる。残り: Mac・iPhone の実機での確認(ここには実機がありません)。翻訳: AI 翻訳(ja/en 以外)、ネイティブの確認はまだです。

### English

Status: a FastAPI-compatible Python framework with a Rust core. Recently CI was made to fetch the sibling RMojo so the compat tests pass, and the RMojo memory limit was documented. On this PC the Rust core is not built, so local checks cover only the pure-Python part. Open: checks on a real Mac and iPhone (no such devices here). Translations: AI-translated (other than ja/en), not yet checked by native speakers.

<!-- directive-20261125-1130 -->
## 2026-11-25 指示(11:30 まで確認不要) / Directive (no confirmation until 11:30)
**日本語**: 2026-11-25 11:30 まで、未了は確認なしで完成まで自動で進める。需要が少ない・必要性が低いと判断される、あるいは1から大規模に作り直す必要があっても、別の方式で済ませず、正面から作り直す。同時に開発する対象: COBOL_to・Java_to・MFastAPI・2MFastAPI(他の言語・フレームワークから MFastAPI 互換へ。Node.js・TypeScript は例外的に選べ、選ばない選択も可能)・RMojo・RPoRea(RPoem と Reaxt の統合)・2RPoRea(他の言語・フレームワークから RPoRea へ)・Reaxt(React+Next.js+Nuxt.js+Nest.js の統合)。1つ変えたら他を確認し、README・CLAUDE・PORTING・POLICY を日英で同じ日に更新する。
**English**: Until 2026-11-25 11:30, carry unfinished work through to completion automatically without asking. Even if demand looks low or a large from-scratch rewrite is needed, do not substitute another approach; do the real work. Developed together: COBOL_to, Java_to, MFastAPI, 2MFastAPI (converts other languages and frameworks to MFastAPI-compatible code; Node.js and TypeScript are an optional exception and choosing neither is allowed), RMojo, RPoRea (RPoem and Reaxt integrated), 2RPoRea (converts other languages and frameworks to RPoRea) and Reaxt (React + Next.js + Nuxt.js + Nest.js integrated). When one changes, check the others and update README, CLAUDE, PORTING and POLICY in Japanese and English the same day.

<!-- directive-20261125-mojo -->
## 2026-11-25 追記: MFastAPI は Mojo から使えること / Addendum: MFastAPI must be usable from Mojo
**日本語**: MFastAPI は **Mojo から使用できる仕様**を保つ(Python 構文・Rust 実装・Mojo 対応、Windows では RMojo の `compat/wsl-bridge` 経由で WSL 上の Mojo を使う)。2MFastAPI の変換結果も Mojo から使えること。現状: `mojo/` は参照実装で、段階B・Cのサーバーは実験的。Mojo からの呼び出しの実機確認は、確認済みの範囲だけを「確認済み」と書く。11:30 まで確認なしで自動で進める。
**English**: MFastAPI keeps the **specification of being usable from Mojo** (Python syntax, Rust core, Mojo support; on Windows the real Mojo in WSL is driven through RMojo's `compat/wsl-bridge`). Output of 2MFastAPI must be usable from Mojo too. Current state: `mojo/` is a reference implementation and its stage B/C servers are experimental. Only what was actually run is called "verified". Continue automatically without asking until 11:30.

## 2026-11-25 追記: 段階2 rmojo-py と mojo_endpoint(runtime) / Stage 2: rmojo-py and mojo_endpoint(runtime)
**日本語**: `rust/rmojo-py`(PyO3、abi3、拡張名 `rmojo_py`)を追加。Mojo風(Python構文)ソースを RMojo で実行する `Handler(source, max_steps, max_ms)` と `call(name, *args)`(None/bool/int/float/str)。専用の大スタックワーカーでコンパイルは1回、**呼び出し中は GIL を手放す**(300ms の無限ループ中も別 Python スレッドが約28回動いたのを確認)。上限超過は `TimeoutError`、その他は `ValueError`、上限後もワーカーは復帰。`mojo_endpoint(..., runtime=None|"mojo"|"rmojo"|"auto", source=, func=, max_steps=, max_ms=)` を追加:"rmojo" は RMojo 実行、"auto" は公式 Mojo 関数が `ImportError` のとき RMojo へ自動切替、上限超過は HTTP 504、使った実行系は応答ヘッダー `x-mfastapi-runtime`(`mojo`/`rmojo`)。`rmojobridge.rs` は RMojo の `Limits.max_ms` 追加に追従。確認: `tests_rmojo_py.py`(全OK)・`tests/mojo_endpoint/test_rmojo_runtime.py` 5件成功・`cargo test --features rmojo --test rmojo_bridge` 成功。**未確認**: FastAPI 実アプリ越しのヘッダー出力(この PC に fastapi/starlette が無く、導入にはダウンロード許可が要る)、Linux/macOS での rmojo_py ビルド、wheel への同梱、リスト/辞書の値、RSS 上限。ビルドは 64bit Python を `PYO3_PYTHON` で指す(PATH 先頭は32bit)。
**English**: Added `rust/rmojo-py` (PyO3, abi3, module `rmojo_py`): `Handler(source, max_steps, max_ms)` and `call(name, *args)` (None/bool/int/float/str) run Mojo-style (Python-syntax) source in RMojo on one big-stack worker (compiled once) and **release the GIL during a call** (another Python thread ticked about 28 times during a 300 ms spin). Limit overrun raises `TimeoutError`, other errors `ValueError`, and the worker recovers. `mojo_endpoint(..., runtime=None|"mojo"|"rmojo"|"auto", source=, func=, max_steps=, max_ms=)` is new: "rmojo" runs in RMojo; "auto" falls back to RMojo when the official Mojo function raises `ImportError`; limit overrun is HTTP 504; the runtime used is reported in the `x-mfastapi-runtime` response header (`mojo`/`rmojo`). `rmojobridge.rs` follows RMojo's new `Limits.max_ms`. Verified: `tests_rmojo_py.py` (all OK), 5 tests in `tests/mojo_endpoint/test_rmojo_runtime.py`, and `cargo test --features rmojo --test rmojo_bridge`. **Not verified**: the header through a real FastAPI app (no fastapi/starlette on this PC; installing needs download permission), rmojo_py builds on Linux/macOS, shipping it in the wheel, list/dict values, an RSS cap. Build with a 64-bit Python via `PYO3_PYTHON` (the first `python` on PATH is 32-bit).

## Versionless API の方針 / Versionless API policy (2026-11-25)
**日本語**: このリポジトリのAPIは **Versionless**(URL・パッケージ名に `v1` 等を付けず、互換性を壊さず足すだけで進化させる)。欄は足すだけ・足す欄は省略可能・知らない欄は無視・名前変更は新名を足して旧を非推奨に(期限は既定で6か月以上、使用0を確認してから削除)・enumは未知の値を許容・安全上の緊急修正は例外。既存の `.v1` は呼び出し側を壊さないよう当面残し、版なしの別名を追加して段階的に畳む。詳細・出典は [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md) 末尾。適用はこれから順に行う。
**English**: This repository's APIs are **Versionless**: no `v1` in URLs or package names; evolve by adding only, without breaking compatibility. Only add fields, new fields are optional, ignore unknown fields, rename by adding the new name and deprecating the old (default deadline 6 months or more, remove only after confirming zero use), tolerate unknown enum values, emergency security fixes are the exception. Existing `.v1` names stay for now so callers do not break; a versionless alias is added and `.v1` is retired in stages. Details and sources: the end of [`runo/API-POLICY.md`](https://github.com/aon-co-jp/runo/blob/main/API-POLICY.md). Rollout is step by step.

## 2026-11-26 追記: FastAPI 実アプリ越しの確認 / Verified through a real FastAPI app
**日本語**: 許可を得て fastapi・httpx を導入し(Python 3.13 64bit)、`tests/mojo_endpoint/test_rmojo_fastapi.py` で実 FastAPI アプリ越しに確認した: `/fib/20`=200・6765・ヘッダー `x-mfastapi-runtime: rmojo`、無限ループ=504、不正パス=422、504後も復帰、`/openapi.json` に経路が載る。全6件成功。**未確認**: Linux/macOS での rmojo_py ビルド、wheel 同梱、リスト/辞書の値、RSS 上限。
**English**: With permission, fastapi and httpx were installed (64-bit Python 3.13) and `tests/mojo_endpoint/test_rmojo_fastapi.py` verified it through a real FastAPI app: `/fib/20` returns 200, 6765 and the header `x-mfastapi-runtime: rmojo`; an infinite loop gives 504; a bad path 422; it recovers after a 504; the route appears in `/openapi.json`. All 6 tests pass. **Not verified**: rmojo_py builds on Linux/macOS, wheel packaging, list/dict values, an RSS cap.
