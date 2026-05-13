import Foundation

struct Snapshot: Decodable {
    let state: String
    let emoji: String
    let correction_rate: Double
    let kpm60: Double
}

public struct DaemonStatus {
    public let running: Bool
    public let state: String
    public let emoji: String
    public let correctionRate: Double
    public let keysPerMin: Int

    public static let notRunning = DaemonStatus(
        running: false, state: "Not running", emoji: "⚫",
        correctionRate: 0, keysPerMin: 0
    )
}

public func decodeStatus(from data: Data) -> DaemonStatus {
    guard let snap = try? JSONDecoder().decode(Snapshot.self, from: data) else {
        return .notRunning
    }
    return DaemonStatus(
        running: true,
        state: snap.state,
        emoji: snap.emoji,
        correctionRate: snap.correction_rate,
        keysPerMin: Int(snap.kpm60.rounded())
    )
}

public func daemonIsRunning(sockPath: String) -> Bool {
    queryDaemon(sockPath: sockPath).running
}

public func queryDaemon(sockPath: String) -> DaemonStatus {
    let fd = socket(AF_UNIX, SOCK_STREAM, 0)
    guard fd >= 0 else { return .notRunning }
    defer { Darwin.close(fd) }
    guard connectUnixSocket(fd: fd, path: sockPath) == 0 else { return .notRunning }

    var data = Data()
    var buf = [UInt8](repeating: 0, count: 1024)
    while true {
        let n = Darwin.recv(fd, &buf, buf.count, 0)
        if n <= 0 { break }
        data.append(contentsOf: buf[..<n])
    }
    return decodeStatus(from: data)
}

public func findDaemonBinary() -> String? {
    if let execPath = ProcessInfo.processInfo.arguments.first {
        let sibling = URL(fileURLWithPath: execPath)
            .resolvingSymlinksInPath()
            .deletingLastPathComponent()
            .appendingPathComponent("pace-coach")
            .path
        if FileManager.default.isExecutableFile(atPath: sibling) {
            return sibling
        }
    }
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: "/usr/bin/which")
    proc.arguments = ["pace-coach"]
    let pipe = Pipe()
    proc.standardOutput = pipe
    try? proc.run()
    proc.waitUntilExit()
    let output = String(data: pipe.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8)?
        .trimmingCharacters(in: .whitespacesAndNewlines)
    return output?.isEmpty == false ? output : nil
}

public func startDaemon(binaryPath: String) {
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: binaryPath)
    proc.arguments = ["start"]
    proc.standardOutput = FileHandle.nullDevice
    proc.standardError = FileHandle.nullDevice
    proc.terminationHandler = { _ in }
    try? proc.run()
}

public func stopDaemon(binaryPath: String) {
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: binaryPath)
    proc.arguments = ["stop"]
    proc.standardOutput = FileHandle.nullDevice
    proc.standardError = FileHandle.nullDevice
    try? proc.run()
    proc.waitUntilExit()
}

private func connectUnixSocket(fd: Int32, path: String) -> Int32 {
    var addr = sockaddr_un()
    addr.sun_family = sa_family_t(AF_UNIX)
    let size = MemoryLayout.size(ofValue: addr.sun_path)
    withUnsafeMutablePointer(to: &addr.sun_path) { ptr in
        path.withCString { src in
            _ = strncpy(UnsafeMutableRawPointer(ptr).assumingMemoryBound(to: CChar.self), src, size)
        }
    }
    return withUnsafePointer(to: &addr) {
        $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
            Darwin.connect(fd, $0, socklen_t(MemoryLayout<sockaddr_un>.size))
        }
    }
}
