import Foundation

public struct Snapshot: Decodable {
    public let emoji: String
}

public func decodeEmoji(from data: Data) -> String {
    (try? JSONDecoder().decode(Snapshot.self, from: data))?.emoji ?? "⚫"
}

public func queryDaemon(sockPath: String) -> String {
    let fd = socket(AF_UNIX, SOCK_STREAM, 0)
    guard fd >= 0 else { return "⚫" }
    defer { Darwin.close(fd) }

    var addr = sockaddr_un()
    addr.sun_family = sa_family_t(AF_UNIX)
    let sunPathSize = MemoryLayout.size(ofValue: addr.sun_path)
    withUnsafeMutablePointer(to: &addr.sun_path) { ptr in
        sockPath.withCString { src in
            _ = strncpy(
                UnsafeMutableRawPointer(ptr).assumingMemoryBound(to: CChar.self),
                src,
                sunPathSize
            )
        }
    }

    let connected = withUnsafePointer(to: &addr) {
        $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
            Darwin.connect(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
        }
    }
    guard connected == 0 else { return "⚫" }

    var data = Data()
    var buf = [UInt8](repeating: 0, count: 1024)
    while true {
        let n = Darwin.recv(fd, &buf, buf.count, 0)
        if n <= 0 { break }
        data.append(contentsOf: buf[..<n])
    }

    return decodeEmoji(from: data)
}
