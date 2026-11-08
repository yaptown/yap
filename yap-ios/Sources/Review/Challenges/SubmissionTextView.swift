import SwiftUI

/// A wrapping translation editor whose Return key submits, rather than adding
/// a newline (SwiftUI's vertical TextField does not reliably call onSubmit).
struct SubmissionTextView: UIViewRepresentable {
    @Binding var text: String
    @Binding var focused: Bool
    var onSubmit: () -> Void
    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeUIView(context: Context) -> UITextView {
        let view = UITextView()
        view.delegate = context.coordinator
        view.font = .preferredFont(forTextStyle: .title3)
        view.adjustsFontForContentSizeCategory = true
        // Bare text on the card; the challenge draws the underline beneath it.
        view.backgroundColor = .clear
        view.textContainerInset = .zero
        view.textContainer.lineFragmentPadding = 0
        view.returnKeyType = .done
        view.isScrollEnabled = false
        view.accessibilityLabel = L("common-translation")
        return view
    }
    func updateUIView(_ view: UITextView, context: Context) {
        context.coordinator.parent = self
        if view.text != text { view.text = text }
        // Change the responder after this update, not during it: doing it here
        // makes UIKit query the hosting view's graph mid-update and fires the
        // delegate that writes `focused`, a cycle SwiftUI can hang on (a
        // watchdog kill seen in 1.0.3 as the app went to the background).
        guard focused != view.isFirstResponder else { return }
        DispatchQueue.main.async { [focused] in
            if focused && !view.isFirstResponder { view.becomeFirstResponder() }
            if !focused && view.isFirstResponder { view.resignFirstResponder() }
        }
    }
    func sizeThatFits(_ proposal: ProposedViewSize, uiView: UITextView, context: Context) -> CGSize? {
        let width = proposal.width ?? 260
        return CGSize(width: width, height: uiView.sizeThatFits(CGSize(width: width, height: .greatestFiniteMagnitude)).height)
    }
    final class Coordinator: NSObject, UITextViewDelegate {
        var parent: SubmissionTextView
        init(_ parent: SubmissionTextView) { self.parent = parent }
        func textViewDidChange(_ textView: UITextView) { parent.text = textView.text }
        func textViewDidBeginEditing(_ textView: UITextView) { parent.focused = true }
        func textViewDidEndEditing(_ textView: UITextView) { parent.focused = false }
        func textView(_ textView: UITextView, shouldChangeTextIn range: NSRange, replacementText text: String) -> Bool {
            if text == "\n" { parent.onSubmit(); return false }
            return true
        }
    }
}
