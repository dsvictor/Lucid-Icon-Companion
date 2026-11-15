import AppKit
import Foundation

func fail(_ message: String) -> Never {
    fputs("Lucide Companion clipboard: \(message)\n", stderr)
    exit(1)
}

let args = CommandLine.arguments
guard args.count == 3 else {
    fail("Expected paths to an SVG file and a PNG file.")
}
let svgURL = URL(fileURLWithPath: args[1])
let pngURL = URL(fileURLWithPath: args[2])
let svgData: Data
let pngData: Data
do {
    svgData = try Data(contentsOf: svgURL)
    pngData = try Data(contentsOf: pngURL)
} catch {
    fail("Could not read clipboard data files: \(error)")
}

let pasteboard = NSPasteboard.general
pasteboard.clearContents()
let svgWritten = pasteboard.setData(svgData, forType: NSPasteboard.PasteboardType("public.svg-image"))
let pngWritten = pasteboard.setData(pngData, forType: .png)
guard svgWritten && pngWritten else {
    fail("NSPasteboard did not accept both SVG and PNG representations.")
}
print("Copied SVG and PNG to the macOS clipboard.")
