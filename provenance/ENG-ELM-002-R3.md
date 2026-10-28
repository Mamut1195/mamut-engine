# Clean-room log · ENG-ELM-002 #41 follow-up (discriminating test)

- Ticket / PR: #41 follow-up (J0); branch `j2/frame-recover-test`
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Contract implemented: element/1 · frame2d.euler_elastic (tests only)
- Permitted sources used: none external.
- Own derivations: test with EI ∈ {1e12, 1e20, 1e30}, L ∈ {3.1, 0.7, 11.3, 2/3} and three u states with double release; requires shear and moment exactly 0. Verified to FAIL without the repair from #41 (EI=1e12, L=3.1: spurious shear 3.05e-5) and pass with it.
- Third-party code copied: none
- Validation: analytical (beam pinned at both ends without load: V=M=0)
- Comparison with oracle: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados para este ticket (docs/CLEAN_ROOM.md §5)."
- Clean-room reviewer: PENDING (J4)
