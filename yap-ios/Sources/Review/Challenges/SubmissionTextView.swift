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
        view.font = .preferredFont(forTextStyle: .body)
        view.adjustsFontForContentSizeCategory = true
        view.backgroundColor = .tertiarySystemGroupedBackground
        view.layer.cornerRadius = 12
        view.textContainerInset = UIEdgeInsets(top: 12, left: 8, bottom: 12, right: 8)
        view.returnKeyType = .done
        view.isScrollEnabled = false
        view.accessibilityLabel = "Translation"
        return view
    }
    func updateUIView(_ view: UITextView, context: Context) {
        context.coordinator.parent = self
        if view.text != text { view.text = text }
        if focused && !view.isFirstResponder { view.becomeFirstResponder() }
        if !focused && view.isFirstResponder { view.resignFirstResponder() }
    }
    func sizeThatFits(_ proposal: ProposedViewSize, uiView: UITextView, context: Context) -> CGSize? {
        let width = proposal.width ?? 260
        return CGSize(width: width, height: max(80, uiView.sizeThatFits(CGSize(width: width, height: .greatestFiniteMagnitude)).height))
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
