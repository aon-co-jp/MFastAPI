# -*- coding: utf-8 -*-
"""段階A+: Mojoで書いた関数を、FastAPIのエンドポイントとしてそのまま登録する橋渡し。

FastAPIは関数のシグネチャ(引数名・型・既定値)から、パス/クエリ/本文の取り出し・検証(Pydantic)・OpenAPI生成を行う。
Mojo拡張の関数にはシグネチャが無いので、宣言したシグネチャを持つ薄いPythonの関数でくるんで渡す。検証・依存関係・OpenAPI・
`/docs` は本家FastAPIのまま、中身の計算だけがMojo。

    import mfastapi_handlers as h
    app.add_api_route("/fib/{n}", mojo_endpoint(h.fib, {"n": int}, returns=int), methods=["GET"])
    app.add_api_route("/primes", mojo_endpoint(h.count_primes, {"n": (int, 1000)}, returns=int))   # クエリ n(既定1000)

制限: Mojo関数の実行中はGILを保持する(同じプロセスの他のPythonスレッドは止まる)。長い計算は別ワーカー/プロセスで。
"""
import inspect

try:  # fastapi があれば、Mojo側の例外をHTTPエラーにする
    from fastapi import HTTPException, Response
except ImportError:  # pragma: no cover
    HTTPException = None
    Response = None

__all__ = ["mojo_endpoint"]


def mojo_endpoint(fn, params, returns=None, name=None, error_status=400, runtime=None, source=None, func=None,
                  max_steps=50_000_000, max_ms=2000):
    """Mojo(など)の関数 fn を FastAPI のエンドポイント関数へ変換する。

    params: {引数名: 型} または {引数名: (型, 既定値)}。順序は fn の位置引数の順。FastAPI の `Query(...)` などを既定値に渡してもよい。
    returns: 戻り値の型(response_modelの推定・OpenAPIの応答スキーマに使われる)。
    fn が例外を投げた(Mojoの raise)場合は HTTP error_status(既定400)、fastapi が無ければ ValueError。

    runtime(段階2): None=従来どおり fn を呼ぶ(公式Mojo)/ "rmojo"=Mojo風ソース `source` の関数 `func`(既定は fn.__name__ か name)を
    RMojo(rmojo_py、GILを手放して実行、max_steps/max_ms で暴走を止める。上限超過は504)で実行 / "auto"=fn が使えなければ(fn が None
    または呼び出しで ImportError)rmojo へ自動で切り替え。使われた実行系は応答ヘッダー x-mfastapi-runtime("mojo" か "rmojo")に出る。"""
    if runtime not in (None, "mojo", "rmojo", "auto"):
        raise ValueError("runtime must be None, 'mojo', 'rmojo' or 'auto'")
    rm = None
    if runtime in ("rmojo", "auto") and source is not None:
        try:
            import rmojo_py
            rm = rmojo_py.Handler(source, max_steps, max_ms)
        except ImportError:
            if runtime == "rmojo":
                raise
    if runtime == "rmojo" and rm is None:
        raise ValueError("runtime='rmojo' needs source=")
    fname = func or name or getattr(fn, "__name__", None)
    names = list(params)
    plist = []
    for n in names:
        spec = params[n]
        if isinstance(spec, tuple):
            ann, default = spec
            plist.append(inspect.Parameter(n, inspect.Parameter.POSITIONAL_OR_KEYWORD, annotation=ann, default=default))
        else:
            plist.append(inspect.Parameter(n, inspect.Parameter.POSITIONAL_OR_KEYWORD, annotation=spec))
    # 既定値のある引数は後ろへ(Pythonの規則)。呼び出し順は names のまま。
    plist.sort(key=lambda p: p.default is not inspect.Parameter.empty)

    def _fail(status, e):
        if HTTPException is not None:
            raise HTTPException(status_code=status, detail=str(e))
        raise ValueError(str(e))

    def endpoint(**kwargs):
        resp = kwargs.pop("_mf_response", None)
        args = [kwargs[n] for n in names]
        used = None
        try:
            if runtime != "rmojo" and fn is not None:
                try:
                    out, used = fn(*args), "mojo"
                except ImportError:
                    if runtime != "auto" or rm is None:
                        raise
            if used is None:
                out, used = rm.call(fname, *args), "rmojo"
        except TimeoutError as e:  # RMojo の LimitError
            _fail(504, e)
        except Exception as e:  # Mojoの raise は Python の例外として届く
            _fail(error_status, e)
        if resp is not None:
            resp.headers["x-mfastapi-runtime"] = used
        return out

    if Response is not None:  # 実行系ヘッダー用(FastAPI が注入する)
        plist.append(inspect.Parameter("_mf_response", inspect.Parameter.KEYWORD_ONLY, annotation=Response))
    endpoint.__signature__ = inspect.Signature(
        plist, return_annotation=returns if returns is not None else inspect.Signature.empty)
    endpoint.__name__ = name or getattr(fn, "__name__", "mojo_endpoint")
    endpoint.__doc__ = getattr(fn, "__doc__", None)
    if returns is not None:
        endpoint.__annotations__ = dict((p.name, p.annotation) for p in plist)
        endpoint.__annotations__["return"] = returns
    return endpoint
