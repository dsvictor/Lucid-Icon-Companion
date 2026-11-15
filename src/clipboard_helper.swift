import AppKit
import Foundation

func fail(_ message: String) -> Never {
    fputs("Lucide Companion clipboard: \(message)\n", stderr)
    exit(1)
}

let args = CommandLine.arguments
guard args.count == 2 else {
    fail("Expected one path to an SVG file.")
}

let svgURL = URL(fileURLWithPath: args[1])
let svgData: Data
do {
    svgData = try Data(contentsOf: svgURL)
} catch {
    fail("Could not read SVG data: \(error)")
}

let pasteboard = NSPasteboard.general
pasteboard.clearContents()
let svgType = NSPasteboard.PasteboardType("public.svg-image")
let textType = NSPasteboard.PasteboardType("public.utf8-plain-text")
let svgWritten = pasteboard.setData(svgData, forType: svgType)
let textWritten = pasteboard.setString(String(decoding: svgData, as: UTF8.self), forType: textType)

guard svgWritten && textWritten else {
    fail("NSPasteboard did not accept the SVG representations.")
}
print("Copied SVG vector data to the macOS clipboard (no PNG).")
