-- mamutdata.lua: lee CSV de datos/<caso>/ y genera tablas y series PGFPlots.
-- Ningún valor se copia a mano: todo sale de los archivos.
--   referencia.csv  analítica propia (herramientas/gen_referencia.py)
--   engine.csv      salida del Engine (results/1 -> CSV), J2
--   opensees.csv    Q2-PLACEHOLDER: SOLO Q2 lo provee; J2 nunca lo lee ni lo llena.
local M = {}
local function readcsv(path)
  local f = io.open(path, "r"); if not f then return nil end
  local hdr, rows = nil, {}
  for line in f:lines() do
    line = line:gsub("\r", "")
    if line ~= "" and not line:match("^#") then
      local t = {}; for c in line:gmatch("[^,]+") do t[#t+1] = c end
      if not hdr then hdr = t else
        local r = {}; for i, h in ipairs(hdr) do r[h] = tonumber(t[i]) end; rows[#rows+1] = r end
    end
  end
  f:close(); return rows
end
local function file(caso, n) return "datos/" .. caso .. "/" .. n .. ".csv" end
local function has(rows) return rows and #rows > 0 end
local function fmt(v) if v == nil then return "---" end return string.format("%.4g", v) end
local function near(rows, xk, x)
  if not rows then return nil end
  for _, r in ipairs(rows) do if r[xk] and math.abs(r[xk]-x) <= 1e-9*math.max(1,math.abs(x)) then return r end end
end
-- Serie PGFPlots: estilo, columnas x/y; si no hay datos, no emite \addplot.
function M.serie(caso, fuente, xk, yk, estilo, leyenda)
  local rows = readcsv(file(caso, fuente))
  if not has(rows) then
    tex.print("\\addlegendimage{" .. estilo .. "}\\addlegendentry{" .. leyenda .. " (pendiente)}")
    return
  end
  local c = {}
  for _, r in ipairs(rows) do if r[xk] and r[yk] then c[#c+1] = "(" .. r[xk] .. "," .. r[yk] .. ")" end end
  tex.print("\\addplot[" .. estilo .. "] coordinates {" .. table.concat(c, " ") .. "};")
  tex.print("\\addlegendentry{" .. leyenda .. "}")
end
-- Tabla de errores: |calc-ref| <= A + R|ref|
function M.tabla(caso, xk, yk, A, R)
  local ref = readcsv(file(caso, "referencia"))
  local eng = readcsv(file(caso, "engine"))
  local ops = readcsv(file(caso, "opensees"))
  if not has(ref) then
    tex.print("\\multicolumn{7}{c}{\\textit{Referencia analítica pendiente de derivar desde el fixture}}\\\\"); return
  end
  for _, r in ipairs(ref) do
    local x, y = r[xk], r[yk]
    local e = near(eng, xk, x); local o = near(ops, xk, x)
    local ev, ov = e and e[yk], o and o[yk]
    local tol = A + R * math.abs(y)
    local err, ver = "---", "\\textit{pendiente}"
    if ev then
      local d = math.abs(ev - y)
      err = string.format("%.2e", d / math.max(math.abs(y), 1e-300))
      ver = (d <= tol) and "\\textcolor{ok}{CUMPLE}" or "\\textcolor{red}{NO CUMPLE}"
    end
    tex.print(fmt(x) .. " & " .. fmt(y) .. " & " .. fmt(ev) .. " & " .. fmt(ov) ..
      " & " .. err .. " & " .. string.format("%.1e", tol) .. " & " .. ver .. "\\\\")
  end
end
function M.estado(caso)
  local s = {}
  for _, n in ipairs({"referencia", "engine", "opensees"}) do
    s[#s+1] = n .. ": " .. (has(readcsv(file(caso, n))) and "con datos" or "vacío")
  end
  tex.print(table.concat(s, "; "))
end
-- Solo referencia analítica y serie del motor. No lee ni imprime otra columna.
function M.estado_par(caso)
  local s = {}
  for _, n in ipairs({"referencia", "engine"}) do
    s[#s+1] = n .. ": " .. (has(readcsv(file(caso, n))) and "con datos" or "vacío")
  end
  tex.print(table.concat(s, "; "))
end
function M.tabla_par(caso, xk, yk, A, R)
  local ref = readcsv(file(caso, "referencia"))
  local eng = readcsv(file(caso, "engine"))
  if not has(ref) then
    tex.print("\\multicolumn{6}{c}{\\textit{Referencia analítica pendiente de derivar}}\\\\"); return
  end
  for _, r in ipairs(ref) do
    local x, y = r[xk], r[yk]
    local e = near(eng, xk, x)
    local ev = e and e[yk]
    local tol = A + R * math.abs(y)
    local err, ver = "---", "\\textit{pendiente}"
    if ev then
      local d = math.abs(ev - y)
      err = string.format("%.2e", d / math.max(math.abs(y), 1e-300))
      ver = (d <= tol) and "\\textcolor{ok}{CUMPLE}" or "\\textcolor{red}{NO CUMPLE}"
    end
    tex.print(fmt(x) .. " & " .. fmt(y) .. " & " .. fmt(ev) ..
      " & " .. err .. " & " .. string.format("%.1e", tol) .. " & " .. ver .. "\\\\")
  end
end
return M
