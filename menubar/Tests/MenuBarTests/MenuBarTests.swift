import XCTest
@testable import MenuBarCore

final class MenuBarTests: XCTestCase {

    func test_decodeStatus_returnsRunningForFullJSON() {
        let json = """
        {"state":"NORMAL","emoji":"🟡","correction_rate":0.032,
         "kpm60":248.0,"kpm_10":250.0,"var_dt":0.05}
        """.data(using: .utf8)!
        let s = decodeStatus(from: json)
        XCTAssertTrue(s.running)
        XCTAssertEqual(s.emoji, "🟡")
        XCTAssertEqual(s.state, "NORMAL")
        XCTAssertEqual(s.correctionRate, 0.032, accuracy: 0.001)
        XCTAssertEqual(s.keysPerMin, 248)
    }

    func test_decodeStatus_returnsNotRunningForEmptyData() {
        XCTAssertFalse(decodeStatus(from: Data()).running)
    }

    func test_decodeStatus_returnsNotRunningForMissingFields() {
        let json = #"{"emoji":"🟡"}"#.data(using: .utf8)!
        XCTAssertFalse(decodeStatus(from: json).running)
    }

    func test_decodeStatus_notRunningHasBlackCircle() {
        XCTAssertEqual(decodeStatus(from: Data()).emoji, "⚫")
    }

    func test_daemonIsRunning_returnsFalseForMissingSocket() {
        XCTAssertFalse(daemonIsRunning(sockPath: "/tmp/no-such-\(UUID()).sock"))
    }

    func test_queryDaemon_returnsNotRunningWhenSocketMissing() {
        let result = queryDaemon(sockPath: "/tmp/no-such-\(UUID()).sock")
        XCTAssertFalse(result.running)
        XCTAssertEqual(result.emoji, "⚫")
    }
}
