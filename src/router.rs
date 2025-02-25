use warp::Filter;
// use std::sync::{Arc, Mutex};

use crate::controllers::me;
// count:Arc<Mutex<u32>>
//web路由定义
pub fn get_router(
  
 ) ->impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone  {
  
  let route_of_get = warp::get().and(
    warp::path!("hello" / String).and_then(move |s: String| me::potplay(s))
    .or(warp::path!("charge").and_then(me::charge))
  );

  let route_of_post = warp::post().and(
    warp::path!("test").and_then(me::test)
  );
  route_of_get.or(route_of_post)
}
