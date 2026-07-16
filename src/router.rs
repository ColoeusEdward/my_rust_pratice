use warp::Filter;
// use std::sync::{Arc, Mutex};
use crate::controllers::me::{self};
// count:Arc<Mutex<u32>>
//web路由定义
// x86_ubuntu 分支：仅保留 /playText 与 /getTextAudio 两个音频接口。
pub fn get_router() -> impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    // 其余 GET 路由（hello/charge/brave/playList/toast）依赖 Windows 专有功能，已移除。
    // let route_of_get = warp::get().and(
    //     warp::path!("hello" / String)
    //         .and_then(move |s: String| me::potplay(s))
    //         .or(warp::path!("charge").and_then(me::charge))
    //         .or(warp::path!("brave").and_then(me::start_barve))
    //         .or(warp::path!("playList").and_then(me::play_list))
    //         .or(warp::path!("toast")
    //             .and(warp::query::<me::ToastQuery>())
    //             .and_then(me::toast_notify)),
    // );

    let route_of_post = warp::post().and(
        warp::path!("playText")
            .and(warp::body::json())
            .and_then(me::play_text)
            // 其余 POST 路由（test/playTextAbogen）依赖被移除的功能，已注释。
            // .or(warp::path!("test").and_then(me::test))
            // .or(warp::path!("playTextAbogen")
            //     .and(warp::body::json())
            //     .and_then(me::play_text_abogen))
            .or(warp::path!("getTextAudio")
                .and(warp::body::json())
                .and_then(me::get_text_audio)),
    );

    let cors = warp::cors()
        // .allow_origin("https://meamoe.top") // 仅允许特定域名
        // .allow_origin("http://localhost:8820") // 也可以允许多个
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "OPTIONS"]) // 明确加上 OPTIONS
        .allow_headers(vec![
            "content-type",
            "authorization",
            "accept",
            "origin",
            "X-Requested-With",
        ])
        .allow_credentials(true)
        .max_age(3600); // 缓存预检请求（Options）的时间，单位为秒

    route_of_post.with(cors)
}
