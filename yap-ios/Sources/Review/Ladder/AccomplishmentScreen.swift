import SwiftUI

struct AccomplishmentScreen: View {
    let view: AccomplishmentView
    let addEvent: (DeckEvent) -> Void
    let onDismiss: () -> Void
    @State private var goalOpen = false
    @State private var confetti = 0
    var body: some View {
        let summary = view.today
        ReviewStepScrollView {
            VStack(spacing: 24) {
                header
                cards(L("accomplishment-screen-new-today"), summary.new_cards)
                cards(L("common-learned"), summary.learned_cards)
                cards(L("accomplishment-screen-back-on-track"), summary.locked_in_cards)
                if !summary.reviewed_words.isEmpty {
                    section(L("accomplishment-reviewed", ["count": summary.reviewed_words.count])) {
                        SentenceFlow(spacing: 6) {
                            ForEach(Array(summary.reviewed_words.enumerated()), id: \.offset) { _, word in
                                Text(word).font(.footnote.weight(.medium)).padding(.horizontal, 8).padding(.vertical, 3)
                                    .overlay { Capsule().strokeBorder(Color.yapBorder) }
                            }
                        }.padding(12).frame(maxWidth: .infinity, alignment: .leading).quietSurface(cornerRadius: 14)
                    }
                }
                LazyVGrid(columns: Self.columns, spacing: 8) {
                    if let recall = summary.recall_percent { stat("\(recall)%", L("accomplishment-screen-recall")) }
                    stat("\(view.words_known)", L("accomplishment-screen-words-known", ["count": view.words_known]))
                }
            }
        } actions: {
            HStack(spacing: 8) {
                Button { confetti += 1 } label: { Image(systemName: "party.popper").frame(minHeight: 22) }
                    .buttonStyle(.bordered).accessibilityLabel(L("accomplishment-screen-celebrate-again"))
                Button(action: onDismiss) { Text(L("common-continue")).frame(maxWidth: .infinity) }
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent)
            }.controlSize(.large)
        }
        .overlay { ConfettiCannons(trigger: confetti) }
        .task {
            let now = Date()
            guard let midnight = Calendar.current.date(byAdding: .day, value: 1, to: Calendar.current.startOfDay(for: now)) else { return }
            do { try await Task.sleep(for: .seconds(midnight.timeIntervalSince(now))) } catch { return }
            onDismiss()
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in if DebugHarness.shared.activeScreen == .review && DebugHarness.shared.command == "next" { onDismiss() } }
        #endif
    }

    private static let columns = [GridItem(.flexible(), spacing: 8), GridItem(.flexible(), spacing: 8)]

    private var header: some View {
        VStack(spacing: 8) {
            Image(systemName: "trophy.fill").font(.system(size: 38)).foregroundStyle(Color.yapCaution)
                .frame(width: 72, height: 72)
                .background(Color.yapCautionSurface, in: RoundedRectangle(cornerRadius: 16, style: .continuous))
                .padding(.bottom, 4)
            Text(view.heading).font(.title2.bold()).multilineTextAlignment(.center)
            Text(view.studied).foregroundStyle(Color.yapMuted).multilineTextAlignment(.center)
            if let streak = view.streak_label {
                Label(streak, systemImage: "flame.fill").font(.subheadline).foregroundStyle(Color.yapMuted)
                    .labelStyle(StreakLabelStyle())
                    .padding(.horizontal, 12).padding(.vertical, 5)
                    .overlay { Capsule().strokeBorder(Color.yapNegative.opacity(0.5)) }
            }
            Button {
                withAnimation(.snappy) { goalOpen.toggle() }
            } label: {
                HStack(spacing: 4) {
                    Text(L("accomplishment-screen-change-daily-goal"))
                    Image(systemName: "chevron.down").font(.caption.weight(.semibold)).rotationEffect(.degrees(goalOpen ? 180 : 0))
                }.font(.subheadline).foregroundStyle(Color.yapMuted).padding(.vertical, 6).contentShape(Rectangle())
            }.buttonStyle(.plain)
            if goalOpen {
                DailyGoalEditor(target: view.target, options: view.goals) { event in
                    addEvent(event)
                    withAnimation(.snappy) { goalOpen = false }
                }
                .id(view.target).transition(.opacity)
            }
        }.frame(maxWidth: .infinity).padding(.top, 16)
    }

    @ViewBuilder private func cards(_ title: String, _ cards: [TodayNewCard]) -> some View {
        if !cards.isEmpty {
            section(title) {
                LazyVGrid(columns: Self.columns, spacing: 8) {
                    ForEach(Array(cards.enumerated()), id: \.offset) { _, card in
                        VStack(alignment: .leading, spacing: 2) {
                            HStack(alignment: .firstTextBaseline, spacing: 4) {
                                Text(card.word).fontWeight(.semibold).lineLimit(1).minimumScaleFactor(0.8)
                                Spacer(minLength: 0)
                                Text(card.card_type.uppercased()).font(.system(size: 9, weight: .medium)).tracking(0.8)
                                    .foregroundStyle(Color.yapMuted.opacity(0.7))
                            }
                            Text(card.translation).font(.subheadline).foregroundStyle(Color.yapMuted).lineLimit(2)
                        }
                        .padding(12).frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                        .cardSurface()
                    }
                }
            }
        }
    }

    private func section<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title.uppercased()).font(.caption.weight(.semibold)).tracking(1).foregroundStyle(Color.yapMuted)
            content()
        }.frame(maxWidth: .infinity, alignment: .leading)
    }

    private func stat(_ value: String, _ label: String) -> some View {
        VStack(spacing: 0) {
            Text(value).font(.title2.bold().monospacedDigit())
            Text(label).font(.caption).foregroundStyle(Color.yapMuted)
        }.padding(12).frame(maxWidth: .infinity).quietSurface(cornerRadius: 14)
    }
}

/// The flame keeps the streak's red even though the words are muted.
private struct StreakLabelStyle: LabelStyle {
    func makeBody(configuration: Configuration) -> some View {
        HStack(spacing: 6) {
            configuration.icon.foregroundStyle(Color.yapNegative)
            configuration.title
        }
    }
}

/// The web's canvas-confetti side cannons: for 0.8s each edge fires paper up
/// and inward; it slows under air drag, drifts down, tumbles and fades. Fires
/// on appear and again whenever `trigger` changes.
private struct ConfettiCannons: View {
    let trigger: Int
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var start: Date?
    @State private var pieces: [Piece] = []

    private struct Piece {
        let fromLeft: Bool
        let delay: Double
        let angle: Double
        let speed: Double
        let spin: Double
        let tumble: Double
        let size: CGSize
        let color: Color
    }

    // canvas-confetti's defaults, so the burst reads the same on both hosts.
    private static let colors: [Color] = [0x26ccff, 0xa25afd, 0xff5e7e, 0x88ff5a, 0xfcff42, 0xffa62d, 0xff36ff].map { hex in
        Color(red: Double(hex >> 16 & 0xff) / 255, green: Double(hex >> 8 & 0xff) / 255, blue: Double(hex & 0xff) / 255)
    }
    private static let lifetime = 3.0
    private static let drag = 6.3 // per second: canvas-confetti's 0.9 velocity decay per frame
    private static let fall = 180.0 // points per second of steady downward drift

    var body: some View {
        TimelineView(.animation(paused: start == nil)) { timeline in
            Canvas { context, size in
                guard let start else { return }
                let elapsed = timeline.date.timeIntervalSince(start)
                for piece in pieces {
                    let t = elapsed - piece.delay
                    guard t > 0, t < Self.lifetime else { continue }
                    let travel = piece.speed / Self.drag * (1 - exp(-Self.drag * t))
                    let x = (piece.fromLeft ? 0 : size.width) + cos(piece.angle) * travel * (piece.fromLeft ? 1 : -1)
                    let y = size.height * 0.6 - sin(piece.angle) * travel + Self.fall * t
                    var layer = context
                    layer.opacity = 1 - t / Self.lifetime
                    layer.translateBy(x: x, y: y)
                    layer.rotate(by: .radians(piece.spin * t))
                    layer.scaleBy(x: 1, y: cos(piece.tumble * t))
                    layer.fill(Path(CGRect(origin: CGPoint(x: -piece.size.width / 2, y: -piece.size.height / 2), size: piece.size)),
                               with: .color(piece.color))
                }
            }
        }
        .allowsHitTesting(false).ignoresSafeArea().accessibilityHidden(true)
        .task(id: trigger) {
            guard !reduceMotion else { return }
            pieces = (0..<288).map { index in
                Piece(fromLeft: index.isMultiple(of: 2), delay: Double(index / 6) / 60,
                      angle: (60 + .random(in: -27.5...27.5)) * .pi / 180, speed: .random(in: 1300...2700),
                      spin: .random(in: -8...8), tumble: .random(in: 4...12),
                      size: CGSize(width: .random(in: 6...9), height: .random(in: 4...6)),
                      color: Self.colors.randomElement()!)
            }
            start = .now
            do { try await Task.sleep(for: .seconds(4)) } catch { return }
            start = nil
        }
    }
}
