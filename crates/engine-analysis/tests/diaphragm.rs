//! Integración de static.linear con T, cargas excéntricas y maestro prescrito.
#[path = "../../engine-assembly/tests/fixtures/counting.rs"]
mod counting;
#[path = "../../engine-assembly/tests/fixtures/rigid_cases.rs"]
mod rigid_cases;

use counting::CountingBackend;
use engine_analysis::StaticLinear;
use engine_contracts::SystemView;
use engine_core::EResult;

#[test]
fn static_linear_recovers_diaphragm_and_projected_equilibrium() -> EResult<()> {
    for settlement in [None, Some(0.03)] {
        let backend = CountingBackend::default();
        let factors = backend.factorizations.clone();
        let solves = backend.solves.clone();
        let mut s = rigid_cases::system(backend, settlement)?;
        let accepted = vec![0.7; s.n_free_dofs()];
        let trial = vec![-0.2; s.n_free_dofs()];
        s.accept(&accepted)?;
        s.internal_force(&trial, &mut vec![0.0; s.n_free_dofs()])?;
        let cases = rigid_cases::cases();
        let results = StaticLinear::run_about(&mut s, &cases, [4.0, -2.0, 1.0])?;
        assert_eq!(results.cases.len(), cases.len());
        assert_eq!(factors.get(), 1);
        assert_eq!(solves.get(), cases.len());
        for (result, (case, factor)) in results.cases.iter().zip(&cases) {
            assert_eq!(result.case, *case);
            assert_eq!(result.factor, *factor);
            assert_eq!(result.free_displacements, s.solve(case, *factor)?);
            rigid_cases::verify_solution(
                &s,
                &result.free_displacements,
                case,
                *factor,
                settlement,
            )?;
            assert_eq!(
                result.global_displacements,
                s.constraints().expand(&result.free_displacements)?
            );
            assert_eq!(
                result.reactions,
                s.reactions(&result.free_displacements, case, *factor)?
            );
            rigid_cases::close(result.free_residual_norm, 0.0);
            rigid_cases::close(result.relative_free_residual, 0.0);
            for &value in &result.equilibrium.residual_force[..3] {
                rigid_cases::close(value, 0.0);
            }
            for value in result.equilibrium.residual_moment {
                rigid_cases::close(value, 0.0);
            }
            rigid_cases::close(result.equilibrium.relative_residual, 0.0);
        }
        assert_eq!(s.accepted_displacement(), accepted);
        assert_eq!(s.trial_displacement(), trial);
    }
    Ok(())
}

#[test]
fn projected_residual_detects_inexact_solver_with_rigid_slaves() -> EResult<()> {
    // Las filas físicas libres del maestro son cero. El error del solver solo
    // aparece en el residual libre al proyectar las fuerzas de los esclavos.
    let backend = CountingBackend {
        solution_scale: 0.5,
        ..CountingBackend::default()
    };
    let mut s = rigid_cases::system(backend, None)?;
    let results = StaticLinear::run(&mut s, &rigid_cases::cases())?;
    for (result, (norm, relative)) in
        results
            .cases
            .iter()
            .zip([(20.0, 0.5), (22.5, 0.5), (0.0, 0.0)])
    {
        rigid_cases::close(result.free_residual_norm, norm);
        rigid_cases::close(result.relative_free_residual, relative);
    }
    Ok(())
}
