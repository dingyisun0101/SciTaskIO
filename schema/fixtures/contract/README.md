# SSTS Contract Fixtures

This fixture set is used to validate the `<series_id>.json` SSTS contract.

## Layout

- `valid/`: inputs expected to pass schema validation.
- `invalid/`: inputs expected to fail schema validation or runtime validation.

## Valid Fixtures

- `valid/minimal.json`
  - Smallest valid shape with one `track_1`.

- `valid/multi_track.json`
  - Multiple tracks with different payload shapes.

- `valid/with_scalars.json`
  - Includes `metadata` + `scalars` and string-valued time labels.

## Invalid Fixtures

- `invalid/bad_track_key.json`
  - Invalid track key (`track_one`), should fail JSON schema key pattern.

- `invalid/missing_label.json`
  - Missing required track field `label`, should fail JSON schema required fields.

- `invalid/times_signal_len_mismatch.json`
  - `len(times) != len(signal)`, should fail runtime validation.

## Validation Notes

JSON Schema file:
- `schema/ssts.json.schema`

Runtime checks still required (not fully expressible in JSON Schema):
- contiguous track numbering (`track_1..track_N`),
- `len(times) == len(signal)`,
- monotonic ordering / deterministic tie-breakers when required by caller.
