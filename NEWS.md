# Changes in 0.1.3

* Add experimental support for embedded-nal-async
  (nightly-only, unstable, guarded by the "async" feature).
* Add `.as_raw_fd()` method on sockets to allows polling them without busy looping.
  The coapclient example was updated to illustrate that.

# Changes in 0.1.2

* Under embedded-nal-tcpextensions, added support for traits in module of the same name.

  This is experimental, but functional and hopefully useful in evaluating that crate.

# Changes in 0.1.1

* Added support for TCP servers (embedded_nal::TcpFullStack).
* Added integration tests.

# Changes in 0.1.0

* embedded-nal dependency changed from 0.2 to 0.6.

  Consequently, all methods now take mutable references.
  The STACK global is still around but deprecated;
  rather than cloning it (which would now become necessary to get a mutable
  version), it should now be constructed through `Stack::default()`.

  Thanks to Ryan Summers for implementing this.

* The MSRV has been incremented to 1.51.0,
  as the underlying embedded-nal version requires that.
