import AppKit
import Foundation
import MenuBarCore

let home = ProcessInfo.processInfo.environment["HOME"] ?? "/tmp"
let sockPath = "\(home)/.pace-coach/pace-coach.sock"

let app = NSApplication.shared
app.setActivationPolicy(.prohibited)

let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
item.button?.title = "⚪"

Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { _ in
    DispatchQueue.global().async {
        let emoji = queryDaemon(sockPath: sockPath)
        DispatchQueue.main.async { item.button?.title = emoji }
    }
}

app.run()
