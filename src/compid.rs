use serde::{Deserialize, Serialize};

// The way requests are handled is as follows:
//
//   1. A component sends a [`ToQueryWorker`] request. The request contains a unique identifying ID.
//   2. [`queryworker`] receives the request, and handles it.
//   3. Once the request is resolved, the request is sent back to a certain component.
//
// This means that the query result needs to be sent to a certain component. Otherwise, the
// response may be propagated by the component that should not have received it. For example, both
// [`PlaylistList`] and [`PlaylistQueue`] may query for the list of media in a playlist. In case of
// [`PlaylistList`], it uses the result to add the entire playlist content to the queue. For
// [`PlaylistQueue`], the result is simply displayed to the user. If the result is sent to both
// components at the same time, unexpected behaviour may occur.
//
// A way around this problem is to give each request-sending components a unique ID. The IDs are
// given in inorder so that we can compare IDs using greater/less-than and narrow down the exact
// component efficiently and without [`.clone()`]ing the responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompID {
    None = 0,
    Home,
    Login,
    MainScreen,
    PlaylistList,
    PlaylistQueue,
    PlayQueue,
    Tasks,
    NowPlaying,
    All,
}
