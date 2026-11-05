#[cfg(feature = "bigo")]
mod tests {
    use acorn_macros::bigo;

    #[test]
    fn test_bigo_compiles_in_no_std_consumer() {
        let cases = trybuild::TestCases::new();
        cases.pass("tests/ui/big-o-no-std.rs");
    }
    #[test]
    fn test_bigo_preserves_log_prefixed_symbols() {
        assert_eq!(bigo!("O(log_count)"), "O(log_count)");
        assert_eq!(bigo!("O(n*log_count)"), "O(n*log_count)");
        assert_eq!(bigo!("O(log_log_a(n))"), "O(log_log_a(n))");
    }
    #[test]
    fn test_bigo_preserves_literal_and_static_type() {
        const TIME: &str = bigo!("O(n log n)");
        const SPACE: &str = bigo!("O(n)");
        let _: &'static str = TIME;
        assert_eq!(TIME, "O(n log n)");
        assert_eq!(SPACE, "O(n)");
    }
    #[test]
    fn test_bigo_accepts_canonical_complexity_corpus() {
        let expressions = [
            bigo!("O(P*B)"),
            bigo!("O(P*(B+T*d))"),
            bigo!("O(R*(P*B^2+B^3))"),
            bigo!("O(I*P*w^2)"),
            bigo!("O(P*log(P))"),
            bigo!("O(P*T)"),
            bigo!("O(P*T^2)"),
            bigo!("O(P*T*(k^2*h+k^2*h^2))"),
            bigo!("O(P*B+G_LUT)"),
            bigo!("O(I*f_p*w^2)"),
            bigo!("O(F_v*I*f_p*w^2)"),
            bigo!("O(F_v*P*K_g)"),
            bigo!("O(A*P*I_m*H)"),
            bigo!("O(I_b*(C_a+X)^3)"),
            bigo!("O(A*C^2*D+A*P*I_m*H)"),
            bigo!("O(N*log(N))"),
            bigo!("O(N^2)"),
            bigo!("O(N*log(N)+N*m)"),
            bigo!("O(N*log(N)+C*(log(N)+m))"),
            bigo!("O(N^2*h+N^3)"),
            bigo!("O(T*N*log(N))"),
            bigo!("O(T*N*log(N)+E_r*T^2)"),
            bigo!("O(T*N^3+N*T*(h_d+h^2))"),
            bigo!("O(I*N*log(N))"),
            bigo!("O(I*N^2)"),
            bigo!("O(I*L_K*N^3)"),
            bigo!("O(P+L_g)"),
            bigo!("O(P*w^2)"),
            bigo!("O(M*P*w^2)"),
            bigo!("O(M*U*P*log(P))"),
            bigo!("O(P^2)"),
            bigo!("O(M*P*w^2+P*M*T+M*T^2+T^3)"),
            bigo!("O(P*log(P))"),
            bigo!("O(P*w)"),
            bigo!("O(P*B^2+B^3)"),
            bigo!("O(P*B*E)"),
            bigo!("O(P*J*(B*E^2+E^3))"),
            bigo!("O(P*I_u*(B*E^2+E^3))"),
            bigo!("O(P*I_u*(B*E^2+E^3)+P)"),
            bigo!("O(T*P*J*(B*E^2+E^3))"),
            bigo!("O(T*P*B)"),
            bigo!("O(T*C^3*D+P*B*T*(h_d+h^2))"),
            bigo!("O(G*w^2)"),
            bigo!("O(G)"),
            bigo!("O(G^3)"),
            bigo!("O(G*log(G))"),
            bigo!("O(I*G)"),
            bigo!("O(G*T)"),
            bigo!("O(G*T^2)"),
            bigo!("O(G*T*(k^2*h+k^2*h^2))"),
            bigo!("O(I*G*log(G))"),
            bigo!("O((V+V^2)*log(V))"),
            bigo!("O(F*log(F)+K+V)"),
            bigo!("O(L_g*(E_g*d+F*d^2))"),
            bigo!("O(F*a)"),
            bigo!("O(F*log(F)+K)"),
            bigo!("O(F^3)"),
            bigo!("O(V*log(V))"),
            bigo!("O(V^2)"),
            bigo!("O(n*c^2)"),
            bigo!("O(H*log(H))"),
            bigo!("O((V+K)*log(V))"),
            bigo!("O(V*log(V)+K)"),
            bigo!("O(L_g*(E_g*d+V*d^2))"),
            bigo!("O(S^3+Q*S^2)"),
            bigo!("O(Q*log(S))"),
            bigo!("O(S*T)"),
            bigo!("O(S*T^2)"),
            bigo!("O(S*T*(h_d+h^2))"),
            bigo!("O(T*q^3)"),
            bigo!("O(q^3+S*T*q^2)"),
            bigo!("O(n*log(n))"),
            bigo!("O(n^2)"),
            bigo!("O(n*(h_d+h^2)+C^1*D)"),
            bigo!("O(n*w*log(w))"),
            bigo!("O(n*log(w))"),
            bigo!("O(P*B+B*polylog(P))"),
            bigo!("O(P*B+R*log(B)*t*H^2/e)"),
            bigo!("O(P*a*q*d*log(1/d))"),
            bigo!("O(P+log(P)^2+P^(1/2))"),
            bigo!("O(P+I*f_p*s^2*k^2*log(w^2)/e)"),
            bigo!("O(P*R_s*G_q)"),
            bigo!("O(P*T+W_out(P))"),
            bigo!("O(P*T*B+P*T*R_s*G_q+W_out(P))"),
            bigo!("O(P*B+B*P^(1/2)+W_out(P))"),
            bigo!("O(P*B+P*w^2)"),
            bigo!("O(C^2*D)"),
            bigo!("O(C^3*D)"),
            bigo!("O(C^2*D+f*m^2*d)"),
            bigo!("O(P*B+B*E)"),
            bigo!("O(P*E^(1/2))"),
            bigo!("O(P*s^2*k^2*log(E)/e)"),
            bigo!("O(P*J^(1/2)*s^2*k^2*log(E)/e)"),
            bigo!("O(P*B+P*R_s*G_q+W_out(P))"),
            bigo!("O(P*B+P*s^2*k^2*log(E)+W_out(P*B))"),
            bigo!("O(T*P*J^(1/2)*s^2*k^2*log(E)/e)"),
            bigo!("O(P*B+log(B)*t*H^2/e)"),
            bigo!("O(P*B+2*log(B)*t*H^2/e)"),
            bigo!("O(G+polylog(G))"),
            bigo!("O(G+G/e+W_out(P*E))"),
            bigo!("O(G+G*R_s*G_q+W_out(G))"),
            bigo!("O(G+log(G)^2+G^(1/2))"),
            bigo!("O(G+log(G)^2+W_out(G))"),
            bigo!("O(G+log_a(G)*k*c^(e-d))"),
            bigo!("O(G*T+polylog(G))"),
            bigo!("O(G*T+W_out(G))"),
            bigo!("O(G*T+G*T*R_s*G_q+W_out(G))"),
            bigo!("O(log_a(G)*k*c^(e-d)+W_out(G))"),
            bigo!("O(G+I*G^(3/2))"),
            bigo!("O(G+G*R_s*G_q+W_out(G))"),
            bigo!("O(V*K^(1/2)+K)"),
            bigo!("O(log(B*F)+W_out(K))"),
            bigo!("O(F+E_g)"),
            bigo!("O(F+E_g+F*R_s*G_q+W_out(F))"),
            bigo!("O((a*F*r)^(1/2)+r)"),
            bigo!("O(log(B*F)+W_out(K))"),
            bigo!("O(V+V^(3/2))"),
            bigo!("O(F^2*R_a*t_a)"),
            bigo!("O(F*R_s*G_q+F^(3/2)+W_out(F))"),
            bigo!("O(T*(F+E_g)+T*F*R_s*G_q+W_out(F))"),
            bigo!("O(V+V*K^(1/2)+K+W_out(K))"),
            bigo!("O(V^(3/2))"),
            bigo!("O(V+E_g+V*R_s*G_q+W_out(V))"),
            bigo!("O(S+(k*S)^(1/2)+W_out(Q))"),
            bigo!("O(S+s^2*k^2*log(S)/e+Q/e+W_out(Q))"),
            bigo!("O(Q*M_b+Q*R_s*G_q+W_out(Q))"),
            bigo!("O(S*T+W_out(S))"),
            bigo!("O(S*T+S*T*R_s*G_q+W_out(S*T))"),
            bigo!("O(S*T*q)"),
            bigo!("O(s^2*k^2*log(q)/e)"),
            bigo!("O(T*k*polylog(q/e)*log(1/e))"),
            bigo!("O(n+n*R_s*G_q+W_out(n))"),
            bigo!("O(n+n*w^(1/2)+W_out(n))"),
            bigo!("O(n^2*m+W_out(n))"),
        ];
        assert_eq!(expressions.len(), 134);
    }
    #[test]
    fn test_bigo_rejects_invalid_inputs() {
        let cases = trybuild::TestCases::new();
        cases.compile_fail("tests/ui/big-o-invalid-functions.rs");
        cases.compile_fail("tests/ui/big-o-invalid-syntax.rs");
        cases.compile_fail("tests/ui/big-o-nonliteral.rs");
    }
}
