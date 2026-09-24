import SwiftUI

struct VoiceActorBanner: View {
    @Environment(AudioPlayer.self) private var audio
    var body: some View {
        if audio.needsAccount {
            let copy = account_copy()
            VStack(alignment: .leading, spacing: 6) {
                Text(copy.audio_needs_account_title).font(.subheadline.bold())
                Text(copy.audio_needs_account_body).font(.footnote)
            }
            .padding(14).background(.regularMaterial, in: RoundedRectangle(cornerRadius: 16))
            .padding().allowsHitTesting(false).accessibilityAddTraits(.updatesFrequently)
        } else if let credit = audio.voiceCredit {
            Label(credit, systemImage: "mic.fill")
                .font(.footnote).padding(14)
                .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 16))
                .padding().allowsHitTesting(false)
                .accessibilityAddTraits(.updatesFrequently)
        }
    }
}
