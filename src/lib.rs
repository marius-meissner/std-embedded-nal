mod udp;
mod conversion;

/// The operating system's network stack, implementing ``embedded_nal::UdpStack`` and others.
///
/// This is most easily accessed using the static ``STACK`` instance.
pub struct Stack {
    // Ensure extensibility. Chances are we won't need it, but can still be relaxed easily.
    _private: ()
}

pub static STACK: Stack = Stack { _private: () };
