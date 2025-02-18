%global crate note
# prevent library files from being installed
%global cargo_install_lib 0

Name:           rust-note
# x-release-please-start-version
Version:        0.1.0
# x-release-please-end-version
Release:        %autorelease
Summary:        Note taking utility

License:        MIT OR APACHE-2.0
URL:            https://github.com/joshuachp/note
Source:         %{url}/releases/download/v%{version}/%{crate}-%{version}.crate
Source:         %{url}/releases/download/v%{version}/%{name}-%{version}-vendor.tar.xz

BuildRequires:  cargo-rpm-macros >= 26

%global _description %{expand:
Cli to manage machines and configurations}

%description %{_description}

%package     -n %{crate}
Summary:        %{summary}
License:        MIT OR APACHE-2.0
# LICENSE.dependencies contains a full license breakdown

%description -n %{crate} %{_description}

%files       -n %{crate}
%license LICENSE-MIT
%license LICENSE-APACHE-2.0
%license LICENSE.dependencies
%license cargo-vendor.txt
%{_bindir}/note
%{bash_completions_dir}/note.bash
%{fish_completions_dir}/note.fish
%{zsh_completions_dir}/_note
%{_mandir}/man1/note*

%prep
%autosetup -n %{crate}-%{version} -p1 -a1
# fix shebangs in vendor
%cargo_prep -v vendor
find ./vendor -type f -executable -name '*.rs' -exec chmod -x '{}' \;

%build
%cargo_build
%{cargo_license_summary}
%{cargo_license} > LICENSE.dependencies
%{cargo_vendor_manifest}

%install
%cargo_install
'%{buildroot}%{_bindir}/note' utils completion bash > note.bash
'%{buildroot}%{_bindir}/note' utils completion fish > note.fish
'%{buildroot}%{_bindir}/note' utils completion zsh > _note
install -Dpm 0644 note.bash -t %{buildroot}%{bash_completions_dir}
install -Dpm 0644 note.fish -t %{buildroot}%{fish_completions_dir}
install -Dpm 0644 _note -t %{buildroot}%{zsh_completions_dir}
mkdir -pm 0755 '%{buildroot}%{_mandir}/man1'
'%{buildroot}%{_bindir}/note' utils manpages "%{buildroot}%{_mandir}/man1"

%check
%cargo_test

%files
%license
%doc

%changelog
%autochangelog

