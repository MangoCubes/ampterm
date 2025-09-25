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
// ID naming is as follows:
//   - Value of 0 represents NONE, and corresponding response does not have any destination
//   - Value of 0xFFFFFFFF represents ALL, a response that should be propagated to all components
//   - <Component Name> represents a response that should be propagated to that component
//   - __<Component Name> is a placeholder ID to specify the range of IDs that should be allocated
//     to a certain component

pub const NONE: u32 = 0x00000000;

pub const HOME: u32 = 0x00000003;

pub const LOGIN: u32 = 0x10000000;
pub const MAINSCREEN: u32 = 0x20000000;
pub const PLAYLISTLIST: u32 = 0x21000000;
pub const __PLAYLISTLIST: u32 = 0x21FFFFFF;
pub const PLAYLISTQUEUE: u32 = 0x22000000;
pub const __PLAYLISTQUEUE: u32 = 0x22FFFFFF;
pub const QUEUELIST: u32 = 0x23000000;
pub const __QUEUELIST: u32 = 0x23FFFFFF;
pub const NOWPLAYING: u32 = 0x23000000;
pub const __NOWPLAYING: u32 = 0x23FFFFFF;
pub const __MAINSCREEN: u32 = 0x2FFFFFFF;
pub const __LOGIN: u32 = 0x1FFFFFFF;

pub const __HOME: u32 = 0xFFFFFFFE;
pub const ALL: u32 = 0xFFFFFFFF;
