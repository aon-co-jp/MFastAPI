#![cfg(feature = "rmojo")]
use mfastapi_server::rmojobridge::rmojo_handler;
use mfastapi_server::*;

fn app() -> App {
    let src = "def fib(n: int) -> int:\n    if n < 2:\n        return n\n    return fib(n - 1) + fib(n - 2)\ndef spin(n: int) -> int:\n    while True:\n        n = n + 1\n    return n\ndef hi(name):\n    return 'hi ' + name\n";
    App::new(vec![
        Route { method: Method::Get, body: false, path: "/fib/{n}", name: "fib", params: vec![Param::path("n", Kind::Int)], returns: Kind::Int, handler: rmojo_handler(src, "fib", vec![("n", Kind::Int)], Kind::Int, 1_000_000) },
        Route { method: Method::Get, body: false, path: "/spin/{n}", name: "spin", params: vec![Param::path("n", Kind::Int)], returns: Kind::Int, handler: rmojo_handler(src, "spin", vec![("n", Kind::Int)], Kind::Int, 10_000) },
        Route { method: Method::Get, body: false, path: "/hi", name: "hi", params: vec![Param::query("name", Kind::Str)], returns: Kind::Str, handler: rmojo_handler(src, "hi", vec![("name", Kind::Str)], Kind::Str, 1000) },
    ])
    .unwrap()
}

#[test]
fn rmojo_routes() {
    let a = app();
    let r = a.handle("GET", "/fib/20", "", b"");
    assert_eq!((r.status, r.body.as_str()), (200, "6765"));
    let r = a.handle("GET", "/hi", "name=ann", b"");
    assert_eq!((r.status, r.body.as_str()), (200, "\"hi ann\""));
    assert_eq!(a.handle("GET", "/spin/1", "", b"").status, 504);
    assert_eq!(a.handle("GET", "/fib/abc", "", b"").status, 422);
    assert_eq!(a.handle("GET", "/fib/20", "", b"").status, 200); // worker still alive after a 504
}
