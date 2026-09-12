"""Length grouping for explicit pronunciation batch requests."""


def length_batches(lengths, max_batch_size=8, max_padded_samples=1_920_000,
                   max_length_ratio=1.25):
    """Return indices sorted by length, bounding padding and sample count.

    A long singleton is allowed; the caller handles its memory requirements.
    """
    if max_batch_size < 1 or max_padded_samples < 1 or max_length_ratio < 1:
        raise ValueError("invalid batch limits")
    if any(n <= 0 for n in lengths):
        raise ValueError("audio lengths must be positive")
    batch = []
    for index in sorted(range(len(lengths)), key=lengths.__getitem__):
        if batch and (len(batch) >= max_batch_size
                      or lengths[index] * (len(batch) + 1) > max_padded_samples
                      or lengths[index] > lengths[batch[0]] * max_length_ratio):
            yield batch
            batch = []
        batch.append(index)
    if batch:
        yield batch

