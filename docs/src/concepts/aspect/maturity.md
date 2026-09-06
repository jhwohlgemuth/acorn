# 🪴 Maturity

## Discussion
ASPECT leverages a simplified mapping from augmented technology readiness levels (TRL). Wikipedia[^wikipedia] defines TRLs as "a method for estimating the maturity of technologies during the acquisition phase of a program. TRLs enable consistent and uniform discussions of technical maturity across different types of technology." The U.S. federal acquisitions community widely uses TRLs to assess the maturity of a particular technology. ASPECT also incorporates work[^lavin] that adapted TRLs to directly address the particular nuances of machine learning systems. Moreover, ASPECT maturity levels work well with the capability maturity model but go beyond maintenance processes and efficiency to characterize a technology's maturity in practice. Specifically, ASPECT's openness and portability attributes create a well-defined understanding of a given technology.

**Maturity** in ASPECT reflects the degree to which a system has been tested, validated, and proven in real-world scenarios, as well as its readiness for deployment and integration into existing workflows. In combination with the other ASPECT attributes, ASPECT's maturity attribute provides a holistic view of the system under consideration, beyond a technology's age or quality.

## Levels
| Maturity Level | TRL Mapping | Description |
|---|---|---|
| Unvalidated | N/A | Technology has not been assessed or lacks sufficient evidence of readiness. |
| Discovery | 0-1 | Early exploration and basic research to establish foundational principles. |
| Concept | 2-3 | Concept is defined and initial feasibility is demonstrated. |
| Development | 4-5 | Technology is built and validated through low- to high-fidelity development. |
| Prototype | 6-8 | Working prototype is demonstrated in a relevant environment through operational integration. |
| Proven | 9 | System is proven through successful real-world operation. |

> [!TIP]
> [Read docs.rs documentation for maturity levels](https://docs.rs/acorn-schema/latest/acorn_schema/schema/research_activity/aspect/maturity/index.html)

[^lavin]: A. Lavin et al., "Technology readiness levels for machine learning systems," Nat Commun, vol. 13, no. 1, p. 6039, Oct. 2022, doi: 10.1038/s41467-022-33128-9.
[^wikipedia]: Technology Readiness Level: [https://en.wikipedia.org/wiki/Technology_readiness_level](https://en.wikipedia.org/wiki/Technology_readiness_level)
![TRLs for ML systems](../../_assets/ml_trl.png)
