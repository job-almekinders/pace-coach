import AppKit
import Foundation
import MenuBarCore

final class MenuController: NSObject {
    private let sockPath: String
    private let configPath: String

    private var statusItem: NSStatusItem!
    private var stateItem: NSMenuItem!
    private var correctionRateItem: NSMenuItem!
    private var keysPerMinItem: NSMenuItem!
    private var startItem: NSMenuItem!
    private var stopItem: NSMenuItem!

    init(home: String) {
        sockPath = "\(home)/.pace-coach/pace-coach.sock"
        configPath = "\(home)/.pace-coach/config.json"
    }

    func setup() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        statusItem.button?.title = "⚪"

        let menu = NSMenu()

        let headerItem = NSMenuItem(title: "PACE COACH", action: nil, keyEquivalent: "")
        headerItem.isEnabled = false
        menu.addItem(headerItem)

        stateItem = NSMenuItem(title: "Not running ⚫", action: nil, keyEquivalent: "")
        stateItem.isEnabled = false
        menu.addItem(stateItem)

        correctionRateItem = NSMenuItem(title: "Correction rate: —", action: nil, keyEquivalent: "")
        correctionRateItem.isEnabled = false
        menu.addItem(correctionRateItem)

        keysPerMinItem = NSMenuItem(title: "Keys / min: —", action: nil, keyEquivalent: "")
        keysPerMinItem.isEnabled = false
        menu.addItem(keysPerMinItem)

        menu.addItem(.separator())

        startItem = NSMenuItem(title: "Start", action: #selector(startTapped), keyEquivalent: "")
        startItem.target = self
        menu.addItem(startItem)

        stopItem = NSMenuItem(title: "Stop", action: #selector(stopTapped), keyEquivalent: "")
        stopItem.target = self
        menu.addItem(stopItem)

        menu.addItem(.separator())

        let configItem = NSMenuItem(title: "Open Config…", action: #selector(openConfig), keyEquivalent: "")
        configItem.target = self
        menu.addItem(configItem)

        menu.addItem(.separator())

        let quitItem = NSMenuItem(title: "Quit", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        menu.addItem(quitItem)

        statusItem.menu = menu

        // Auto-start daemon if not already running
        DispatchQueue.global().async {
            if !daemonIsRunning(sockPath: self.sockPath), let binary = findDaemonBinary() {
                startDaemon(binaryPath: binary)
                DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { self.refresh() }
            }
        }
    }

    func refresh() {
        DispatchQueue.global().async {
            let status = queryDaemon(sockPath: self.sockPath)
            DispatchQueue.main.async { self.apply(status) }
        }
    }

    private func apply(_ status: DaemonStatus) {
        statusItem.button?.title = status.emoji
        if status.running {
            stateItem.title = "\(status.state) \(status.emoji)"
            correctionRateItem.title = String(format: "Correction rate: %.1f%%", status.correctionRate * 100)
            keysPerMinItem.title = "Keys / min: \(status.keysPerMin)"
        } else {
            stateItem.title = "Not running ⚫"
            correctionRateItem.title = "Correction rate: —"
            keysPerMinItem.title = "Keys / min: —"
        }
        startItem.isEnabled = !status.running
        stopItem.isEnabled = status.running
    }

    @objc private func startTapped() {
        DispatchQueue.global().async {
            guard let binary = findDaemonBinary() else {
                DispatchQueue.main.async {
                    let alert = NSAlert()
                    alert.messageText = "pace-coach not found"
                    alert.informativeText = "The pace-coach binary was not found next to this app or on PATH."
                    alert.runModal()
                }
                return
            }
            startDaemon(binaryPath: binary)
            DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { self.refresh() }
        }
    }

    @objc private func stopTapped() {
        DispatchQueue.global().async {
            guard let binary = findDaemonBinary() else { return }
            stopDaemon(binaryPath: binary)
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { self.refresh() }
        }
    }

    @objc private func openConfig() {
        let url = URL(fileURLWithPath: configPath)
        if !FileManager.default.fileExists(atPath: configPath) {
            let defaults = """
            {
              "correction_rate_threshold": 0.06,
              "stress_duration_secs": 10,
              "nudge_cooldown_secs": 60
            }
            """
            try? defaults.write(to: url, atomically: true, encoding: .utf8)
        }
        NSWorkspace.shared.open(url)
    }
}

let home = ProcessInfo.processInfo.environment["HOME"] ?? "/tmp"

let app = NSApplication.shared
app.setActivationPolicy(.prohibited)

let controller = MenuController(home: home)
controller.setup()
controller.refresh()

Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { _ in
    controller.refresh()
}

app.run()
