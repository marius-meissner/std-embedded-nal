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
