{
  "schema": "fixture/1",
  "id": "<identificador-propio>",
  "source": {
    "kind": "analytical",
    "citation": "<author, title, date, local note and equations>",
    "derivation": "<assumptions, boundary conditions, SI parameters, equations and own evaluation>"
  },
  "input": {
    "schema": "engine-input/1"
  },
  "expected": [
    {
      "pointer": "<actual numerical JSON Pointer in results/1>",
      "value": 0.0
    }
  ],
  "tolerance": {
    "relative": 1e-10,
    "absolute": 1e-10,
    "justification": "<justify for the units and approximations of this case>"
  },
  "provenance": {
    "ticket": "<ticket>",
    "note": "<relative path to own note>"
  }
}
