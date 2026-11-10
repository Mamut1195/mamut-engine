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

-- Tabla semáforo. Lee tolerancias.csv y las series referencia/engine.
-- No abre datos/<caso>/opensees.csv ni ningún archivo de contraste.
local function parse_csv_line(line)
  local fields, buf, inq = {}, {}, false
  local i = 1
  while i <= #line do
    local c = line:sub(i, i)
    if inq then
      if c == '"' then
        if line:sub(i + 1, i + 1) == '"' then
          buf[#buf + 1] = '"'
          i = i + 1
        else
          inq = false
        end
      else
        buf[#buf + 1] = c
      end
    else
      if c == '"' then
        inq = true
      elseif c == ',' then
        fields[#fields + 1] = table.concat(buf)
        buf = {}
      else
        buf[#buf + 1] = c
      end
    end
    i = i + 1
  end
  fields[#fields + 1] = table.concat(buf)
  return fields
end

local function texesc(s)
  local out = {}
  for i = 1, #s do
    local c = s:sub(i, i)
    if c == "\\" then out[#out + 1] = "\\textbackslash{}"
    elseif c == "%" then out[#out + 1] = "\\%"
    elseif c == "_" then out[#out + 1] = "\\_"
    elseif c == "#" then out[#out + 1] = "\\#"
    elseif c == "&" then out[#out + 1] = "\\&"
    elseif c == "$" then out[#out + 1] = "\\$"
    else out[#out + 1] = c end
  end
  return table.concat(out)
end

local function load_tol()
  local map = {}
  local f = io.open("preregistro/tolerancias.csv", "r")
  if not f then return map end
  local header = nil
  for line in f:lines() do
    line = line:gsub("\r", "")
    if line ~= "" and not line:match("^#") then
      local cells = parse_csv_line(line)
      if not header then
        header = {}
        for i, h in ipairs(cells) do header[h] = i end
      else
        local caso = cells[header.caso]
        map[caso] = {
          caso = caso,
          criterio_aceptacion = cells[header.criterio_aceptacion] or "",
          tolerancia_relativa = cells[header.tolerancia_relativa] or "",
          tolerancia_absoluta = cells[header.tolerancia_absoluta] or "",
        }
      end
    end
  end
  f:close()
  return map
end

local function read_series(path)
  local rows = {}
  local f = io.open(path, "r")
  if not f then return rows end
  for line in f:lines() do
    line = line:gsub("\r", "")
    if line ~= "" and not line:match("^#") then
      local cells = parse_csv_line(line)
      local x, y = tonumber(cells[1]), tonumber(cells[2])
      if x and y then rows[#rows + 1] = {x = x, y = y} end
    end
  end
  f:close()
  return rows
end

local function match_y(eng, x)
  for _, row in ipairs(eng) do
    if math.abs(row.x - x) <= 1e-9 * math.max(1, math.abs(x)) then return row.y end
  end
  return nil
end

local function tol_text(rec)
  local a = rec.tolerancia_absoluta or ""
  local r = rec.tolerancia_relativa or ""
  local function escrito(x) return x ~= "" and x ~= "DESCONOCIDO" end
  if rec.caso == "VAL-035" then
    return texesc(rec.criterio_aceptacion or "") .. " Absoluta " .. texesc(a) .. "."
  end
  if escrito(a) and escrito(r) then
    return "A=" .. texesc(a) .. ", R=" .. texesc(r)
  end
  if a == "DESCONOCIDO" or r == "DESCONOCIDO" then return "DESCONOCIDO" end
  local base = texesc(rec.criterio_aceptacion or "DESCONOCIDO")
  if escrito(a) and r == "" then
    if not (rec.criterio_aceptacion or ""):find(a, 1, true) then
      return base .. " Absoluta " .. texesc(a) .. "."
    end
  end
  return base
end

local function evaluar(caso, rec)
  local ref = read_series("datos/" .. caso .. "/referencia.csv")
  local eng = read_series("datos/" .. caso .. "/engine.csv")
  if #ref == 0 or #eng == 0 then
    return "\\textit{pendiente}", "pendiente", "DESCONOCIDO"
  end
  local a = rec.tolerancia_absoluta or ""
  local r = rec.tolerancia_relativa or ""
  local A, R, judged = nil, nil, false
  if tonumber(a) and tonumber(r) then
    A, R, judged = tonumber(a), tonumber(r), true
  elseif tonumber(a) and r == "" then
    -- Criterio solo absoluto. No se escribe una relativa nueva en el CSV.
    A, R, judged = tonumber(a), 0, true
  end
  local maxe, n, any_zero, falla = nil, 0, false, false
  for _, row in ipairs(ref) do
    local ev = match_y(eng, row.x)
    if not ev then return "\\textit{pendiente}", "pendiente", "DESCONOCIDO" end
    n = n + 1
    local d = math.abs(ev - row.y)
    if row.y == 0 then
      any_zero = true
    else
      local e = d / math.abs(row.y)
      if not maxe or e > maxe then maxe = e end
    end
    if judged then
      local tol = A + R * math.abs(row.y)
      if d > tol then falla = true end
    end
  end
  if n == 0 then return "\\textit{pendiente}", "pendiente", "DESCONOCIDO" end
  local err = "DESCONOCIDO"
  if not any_zero and maxe then err = string.format("%.3e", maxe):gsub(",", ".") end
  local ver = "\\textit{pendiente}"
  if judged and not any_zero then
    ver = falla and "\\textcolor{red}{FALLA}" or "\\textcolor{ok}{PASA}"
  end
  return ver, err, tostring(n)
end

function M.semaforo()
  local map = load_tol()
  for i = 1, 41 do
    local caso = string.format("VAL-%03d", i)
    local rec = map[caso] or {
      caso = caso,
      criterio_aceptacion = "DESCONOCIDO",
      tolerancia_relativa = "DESCONOCIDO",
      tolerancia_absoluta = "DESCONOCIDO",
    }
    local ver, err, n = evaluar(caso, rec)
    tex.print(caso .. " & " .. ver .. " & " .. err .. " & " .. tol_text(rec) .. " & " .. n .. " \\\\")
  end
end

return M
