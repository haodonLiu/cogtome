!include "MUI2.nsh"

Name "COGTOME"
OutFile "target\cogtome-${VERSION}-setup.exe"
InstallDir "$LOCALAPPDATA\Cogtome"
RequestExecutionLevel user

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Install"
    SetOutPath "$INSTDIR"
    File "target\release\cogtome.exe"
    File /r "target\windows-staging\skills"
    File /r "target\windows-staging\units"
    File /r "target\windows-staging\assemblies"
    File "target\windows-staging\cogtome.toml"

    # Create uninstaller
    WriteUninstaller "$INSTDIR\uninstall.exe"

    # Add to PATH (user-level)
    EnVar::AddValue "PATH" "$INSTDIR"

    # Start menu shortcut
    CreateDirectory "$SMPROGRAMS\COGTOME"
    CreateShortCut "$SMPROGRAMS\COGTOME\COGTOME.lnk" "$INSTDIR\cogtome.exe"
    CreateShortCut "$SMPROGRAMS\COGTOME\Uninstall.lnk" "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\cogtome.exe"
    RMDir /r "$INSTDIR\skills"
    RMDir /r "$INSTDIR\units"
    RMDir /r "$INSTDIR\assemblies"
    Delete "$INSTDIR\cogtome.toml"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    RMDir /r "$SMPROGRAMS\COGTOME"
    EnVar::DeleteValue "PATH" "$INSTDIR"
SectionEnd
