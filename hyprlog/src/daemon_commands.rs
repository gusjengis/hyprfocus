use std::os::unix::net::UnixDatagram;

// send command to daemon using datagram
pub fn send_command(command: &str) {
    const SOCKET_PATH: &str = "/tmp/hyprlog.sock";
    if let Ok(sock) = UnixDatagram::unbound() {
        let _ = sock.send_to(command.as_bytes(), SOCKET_PATH);
    }
}
