use acorn::analyzer::discovery::{RemoteEntity, RemoteOrganizationRole};
use clap::ValueEnum;
use derive_more::Display;

/// DOE CODE result view used by gather.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, ValueEnum)]
pub(crate) enum SearchView {
    /// Software projects.
    Projects,
    /// Developers and contributors.
    People,
    /// Credited organizations.
    Organizations,
}
impl From<&SearchView> for RemoteEntity {
    fn from(value: &SearchView) -> Self {
        match value {
            | SearchView::Projects => Self::Project,
            | SearchView::People => Self::Person,
            | SearchView::Organizations => Self::Organization,
        }
    }
}
/// Organization relationship used by DOE CODE searches.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq, ValueEnum)]
pub(crate) enum OrganizationRole {
    /// Any credited organization relationship.
    Any,
    /// Submitting site.
    SiteOwner,
    /// Research organization.
    Research,
    /// Sponsoring organization.
    Sponsor,
    /// Contributing organization.
    Contributor,
    /// Developing organization.
    Developer,
}
impl From<&OrganizationRole> for RemoteOrganizationRole {
    fn from(value: &OrganizationRole) -> Self {
        match value {
            | OrganizationRole::Any => Self::Any,
            | OrganizationRole::SiteOwner => Self::SiteOwner,
            | OrganizationRole::Research => Self::Research,
            | OrganizationRole::Sponsor => Self::Sponsor,
            | OrganizationRole::Contributor => Self::Contributor,
            | OrganizationRole::Developer => Self::Developer,
        }
    }
}
/// DOE national laboratory used to scope DOE CODE searches.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum DoeLab {
    /// [Ames National Laboratory](https://www.ameslab.gov/).
    Ames,
    /// [Argonne National Laboratory](https://www.anl.gov/).
    Anl,
    /// [Brookhaven National Laboratory](https://www.bnl.gov/).
    Bnl,
    /// [Fermi National Accelerator Laboratory](https://www.fnal.gov/).
    Fnal,
    /// [Idaho National Laboratory](https://inl.gov/).
    Inl,
    /// [Lawrence Berkeley National Laboratory](https://www.lbl.gov/).
    Lbnl,
    /// [Lawrence Livermore National Laboratory](https://www.llnl.gov/).
    Llnl,
    /// [Los Alamos National Laboratory](https://www.lanl.gov/).
    Lanl,
    /// [National Energy Technology Laboratory](https://www.netl.doe.gov/).
    Netl,
    /// [National Renewable Energy Laboratory](https://www.nrel.gov/).
    Nrel,
    /// [Oak Ridge National Laboratory](https://www.ornl.gov/).
    Ornl,
    /// [Pacific Northwest National Laboratory](https://www.pnnl.gov/).
    Pnnl,
    /// [Princeton Plasma Physics Laboratory](https://www.pppl.gov/).
    Pppl,
    /// [Sandia National Laboratories](https://www.sandia.gov/).
    Snl,
    /// [Savannah River National Laboratory](https://www.srnl.gov/).
    Srnl,
    /// [SLAC National Accelerator Laboratory](https://www.slac.stanford.edu/).
    Slac,
    /// [Thomas Jefferson National Accelerator Facility](https://www.jlab.org/).
    Tjnaf,
}
impl DoeLab {
    /// Return the DOE laboratory acronym used by DOE CODE.
    pub(crate) const fn code(self) -> &'static str {
        match self {
            | Self::Ames => "AMES",
            | Self::Anl => "ANL",
            | Self::Bnl => "BNL",
            | Self::Fnal => "FNAL",
            | Self::Inl => "INL",
            | Self::Lbnl => "LBNL",
            | Self::Llnl => "LLNL",
            | Self::Lanl => "LANL",
            | Self::Netl => "NETL",
            | Self::Nrel => "NREL",
            | Self::Ornl => "ORNL",
            | Self::Pnnl => "PNNL",
            | Self::Pppl => "PPPL",
            | Self::Snl => "SNL",
            | Self::Srnl => "SRNL",
            | Self::Slac => "SLAC",
            | Self::Tjnaf => "TJNAF",
        }
    }
}
impl core::fmt::Display for DoeLab {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.code())
    }
}
