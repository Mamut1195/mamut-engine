{
  "schema": "fixture/1",
  "id": "<identificador-propio>",
  "source": {
    "kind": "analytical",
    "citation": "<autor, título, fecha, nota local y ecuaciones>",
    "derivation": "<hipótesis, contorno, parámetros SI, ecuaciones y evaluación propia>"
  },
  "input": {
    "schema": "engine-input/1"
  },
  "expected": [
    {
      "pointer": "<JSON Pointer numérico real en results/1>",
      "value": 0.0
    }
  ],
  "tolerance": {
    "relative": 1e-10,
    "absolute": 1e-10,
    "justification": "<justificar para las unidades y aproximaciones de este caso>"
  },
  "provenance": {
    "ticket": "<ticket>",
    "note": "<ruta relativa a nota propia>"
  }
}
