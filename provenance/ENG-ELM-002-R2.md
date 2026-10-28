# Clean-room log · ENG-ELM-002 J4 r2 follow-up (recover.rs)

- Ticket / PR: J4 LOW finding on #31; branch `j2/frame-recover-spurious`
- Author(s): J2 (agent, Grok Bot) · 2026-10-11 (America/Guayaquil)
- Role: implementer
- Contract implemented: element/1 · frame2d.euler_elastic (recover)
- Permitted sources used: none external; with double release the condensed bending is exactly zero.
- Own derivations: with double release the elastic bending part (indices 1,2,4,5) is set to zero instead of calculating K·u_recuperado, which with EI=1e20 only contributed rounding noise (shear ±64 N, M=198 N·m with L=3.1). Shear/moment remain exactly those of the load (±qL/2, qL²/8).
- Third-party code copied: none
- Validation: analytical (pinned beam with uniform load; unloaded: shear and moment exactly 0)
- Comparison with oracle: no
- Declaration: "No consulté código fuente de OpenSees, Stabileo ni derivados para este ticket."
- Clean-room reviewer: PENDING (J4)
