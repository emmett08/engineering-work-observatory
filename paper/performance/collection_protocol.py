"""Pure row validation shared by the successor collector and summariser."""

from run_benchmarks import HEADER, QS, REPETITIONS, SEEDS

TIMINGS = ("scan_ns", "index_build_ns", "index_query_ns", "index_drop_ns", "index_total_ns")


def validate_row(row):
    """Return a typed protocol row, rejecting malformed or incompatible values."""
    if set(row) != set(HEADER):
        raise ValueError("Unexpected benchmark result columns")
    try:
        values = {field: int(row[field]) for field in HEADER if field != "first"}
    except (TypeError, ValueError) as error:
        raise ValueError("Benchmark numeric fields must be integers") from error
    seed, rep, q = values["seed"], values["rep"], values["q"]
    if seed not in SEEDS or q not in QS or rep not in range(REPETITIONS):
        raise ValueError("Benchmark key is outside the declared protocol")
    first = "scan" if (SEEDS.index(seed) + rep + QS.index(q)) % 2 == 0 else "index"
    expected = {"accepted_count": 100_000, "labour_count": 400_000, "delta_hours": 720}
    if row["first"] != first or any(values[key] != value for key, value in expected.items()):
        raise ValueError("Benchmark result does not match the declared protocol")
    for field in (*TIMINGS, "checksum"):
        if not 0 <= values[field] < 2**128:
            raise ValueError(f"Benchmark value is outside the unsigned 128-bit range: {field}")
    if values["scan_ns"] == 0 or values["index_total_ns"] == 0:
        raise ValueError("Measured batch times must be positive")
    values["first"] = first
    return values
