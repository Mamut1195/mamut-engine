# Axis and sign convention · ENG-CORE-003

**APPROVED by Jonathan 2026-10-10 22:52: X-Y horizontal, Z vertically upward (the Engine governs)**

**Status: APPROVED** (Jonathan, 2026-10-10 22:52): X-Y horizontal, Z vertically upward; the Engine governs the convention. This text and the `engine-model` rustdoc
record that decision. The 2D elevation convention remains pending (see “Pending”).
Original text based on standard structural mechanics.

## Global axes and dimension

X, Y and Z form a right-handed triad: X × Y = Z. Z is vertically upward.
Planar D2 models are represented in X-Y, with Z = 0 and components Ux, Uy and
Rz. Therefore, the D2 plane is horizontal; a vertical frame would require
D3 or an explicit decision on another plane (see “Pending”). Y is not interpreted as
vertical. D3 supports Ux, Uy, Uz, Rx, Ry and Rz.

Each node declares `Dim`. The domain model allows nodes of both
dimensions to be stored, without resolving connection compatibility in this ticket.
Coordinates have three components; the D2 condition Z = 0 remains an
input convention and is not checked geometrically here. DOFs of nodes,
supports, actions and masses are checked against the node dimension.

## Local member axes

End i is the first node and j the second. The local unit x axis points
from i to j. In D2, y = (-x_y, x_x, 0) and z = x × y, so z points
toward +Z. In D3, the x direction alone cannot determine the triad: the element receives an
orientation vector that is the **local z axis reference** (x-z plane). It is normalized, orthogonalized with respect to x (z' = v − (v·x)x), and
y = z × x is formed, making (x, y, z) right-handed. For 2D, z = +Z
and y = Z × x agrees with the D2 convention. A vertical column (x = ±Z) rejects the +Z orientation because it is parallel to x: another
vector is required (e.g. +X or +Y). A zero orientation or one nearly parallel to
x (sine of the angle ≤ 1e-8) is rejected. This crate only stores the domain and does not create
an automatic spatial orientation. Reversing i and j changes the local orientation;
it does not change load directions defined in global axes.

## Actions, displacements and rotations

A positive nodal force or displacement follows its positive global axis.
Positive moments and rotations follow the right-hand rule about
the indicated axis. In D2, Rz and a positive nodal moment about Z are
counterclockwise when viewed from +Z toward the origin. A load on Ux, Uy or Uz is
a force; a load on Rx, Ry or Rz is a moment.

Internal resultants are expressed in local axes. Positive N means
tension: the normal forces on the two faces pull outward from the segment.
Masses associated with translations are lumped masses; those associated with rotations
are rotational inertias. All inputs must use consistent units;
the domain does not impose a unit system.

## Signs by section face and member end

A positive face has outward normal +x and a negative face has outward
normal -x. The following symbols are signed internal resultants, rather than
always-positive magnitudes. For a positive value, these directions apply:

| Resultant | Positive face (end j) | Negative face (end i) |
| --- | --- | --- |
| Axial N | Force +x | Force -x |
| Shear Vy | Force +y | Force -y |
| Shear Vz | Force +z | Force -z |
| Torsion T | Moment about +x | Moment about -x |
| Bending My | Moment about +y | Moment about -y |
| Bending Mz | Moment about +z | Moment about -z |

Each moment direction in the table follows the right-hand rule.
Only N, Vy and Mz apply in D2. For example, positive Vy points toward -y
at i and toward +y at j; positive Mz is clockwise at i and counterclockwise at j when viewed
from +z. At an interior section, forces and moments on opposite faces
are equal and opposite when comparing the same section. The table expresses a
sign convention; it does not require resultants at i and j to have the same
value in the presence of distributed loads.

Global nodal loads and these face resultants are distinct concepts.
A future assembly must declare whether it provides actions on the member or
actions of the member on the node: switching between them reverses the signs.
This ticket does not calculate or transform those actions.

## Geometric tolerance and inspection scope

`LENGTH_TOLERANCE = 1e-12` is absolute, in the unit used for coordinates.
A three-component Euclidean distance less than or equal to that value is reported
as zero length (`ENG-E-0005`). The distance is accumulated with `hypot` to avoid
overflow of squares and loss of small distances when squaring.
Length is not checked if ends are missing or their coordinates
are nonfinite; those defects have their own diagnostics.

`validate` does not perform structural analysis or check stability, positive
stiffnesses, dimensional compatibility between nodes or required properties.
Its diagnostics accumulate in deterministic order and reuse the
`engine-core` catalog. Insertion neither validates nor removes duplicates.

## Pending

**DECISION PENDING.** The 2D elevation, section or inclined-plane convention
(the x2D→X, y2D→Z mapping that `contracts` will declare) is undefined: the plane
normal, Rz direction and gravity are missing. The following is only an **unapproved
proposal**:

> 2D elevation plane: x2D→X, y2D→Z; plane normal -Y; Rz→rotation about Y,
> with sign following the right-hand rule (positive rotation from +X toward +Z according to
> `contracts`); gravity -Z.

Until a decision is made, D2 models in this crate remain in the horizontal X-Y plane.
