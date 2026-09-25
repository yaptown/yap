
// Appended to the generated runtime by check.py so private cache machinery
// can be checked without adding test hooks to the production API.
@MainActor func testStableCache() throws {
    func result(_ lo: UInt64, hi: UInt64 = 0, status: UInt32 = 0) -> BridgeResult {
        BridgeResult(handle: nil, value: 0, status: status,
                     data: BridgeBuffer(data: nil, len: 0), hash_lo: lo, hash_hi: hi)
    }
    var decodes = 0
    func decode(_ result: BridgeResult) throws -> UInt64 {
        decodes += 1
        defer { bridgerton_buffer_free(result.data) }
        try bridgeCheck(result)
        return result.hash_lo
    }
    let bounded = BridgeStableCache<UInt64>(strong: false)
    _ = try bounded.receive(result(1), decode: decode)
    _ = try bounded.receive(result(1), decode: decode)
    precondition(decodes == 1, "cache hit bypasses decoding")
    _ = try bounded.receive(result(1, hi: 1), decode: decode)
    precondition(decodes == 2, "both hash halves participate in lookup")
    for i in 2...16 { _ = try bounded.receive(result(UInt64(i)), decode: decode) }
    _ = try bounded.receive(result(1), decode: decode)
    precondition(decodes == 18, "default cache evicts after 16 distinct values")
    let strong = BridgeStableCache<UInt64>(strong: true)
    for i in 0..<32 { _ = try strong.receive(result(UInt64(i)), decode: decode) }
    let before = decodes
    for i in 0..<32 { _ = try strong.receive(result(UInt64(i)), decode: decode) }
    precondition(decodes == before, "strong cache does not evict")
    for _ in 0..<2 {
        do { _ = try strong.receive(result(0, status: 2), decode: decode); fatalError("expected error") }
        catch is BridgeError {}
    }
    precondition(decodes == before + 2, "errors never hit the cache")
    let optional = BridgeStableCache<UInt64?>(strong: false)
    for _ in 0..<2 {
        let value = try optional.receive(result(0)) { result in
            bridgerton_buffer_free(result.data)
            decodes += 1
            return nil
        }
        precondition(value == nil)
    }
    precondition(decodes == before + 3, "nil is a cached value")
    print("PASS: Swift cache decoding, 128-bit keys, bounded eviction, strong retention, errors, and nil")
}
