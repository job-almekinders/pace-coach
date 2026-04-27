import XCTest
@testable import MenuBarCore

final class MenuBarTests: XCTestCase {
    func testDecodeValidEmoji() throws {
        let json = #"{"emoji":"🟡"}"#.data(using: .utf8)!
        XCTAssertEqual(decodeEmoji(from: json), "🟡")
    }

    func testDecodeAllStateEmojis() throws {
        for emoji in ["⚪", "🔵", "🟡", "🔴"] {
            let json = "{\"emoji\":\"\(emoji)\"}".data(using: .utf8)!
            XCTAssertEqual(decodeEmoji(from: json), emoji)
        }
    }

    func testDecodeFallsBackOnInvalidJSON() {
        let bad = "not json".data(using: .utf8)!
        XCTAssertEqual(decodeEmoji(from: bad), "⚫")
    }

    func testDecodeFallsBackOnEmptyData() {
        XCTAssertEqual(decodeEmoji(from: Data()), "⚫")
    }

    func testDecodeFallsBackOnMissingEmojiField() {
        let json = #"{"state":"NORMAL"}"#.data(using: .utf8)!
        XCTAssertEqual(decodeEmoji(from: json), "⚫")
    }

    func testSnapshotDecodable() throws {
        let json = #"{"emoji":"🔴","extra_field":"ignored"}"#.data(using: .utf8)!
        let snap = try JSONDecoder().decode(Snapshot.self, from: json)
        XCTAssertEqual(snap.emoji, "🔴")
    }

    func testQueryDaemonFallsBackWhenSocketMissing() {
        // Uses a path that definitely does not exist
        let result = queryDaemon(sockPath: "/tmp/pace-coach-no-such-socket-\(UUID().uuidString).sock")
        XCTAssertEqual(result, "⚫")
    }
}
