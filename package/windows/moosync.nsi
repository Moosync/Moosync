!include "MUI2.nsh"

Name "Moosync"
OutFile "MoosyncSetup.exe"
InstallDir "$PROGRAMFILES64\Moosync"
RequestExecutionLevel admin

!define MUI_ABORTWARNING

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "Install"
    SetOutPath $INSTDIR
    
    ; The variable ${BINARY_PATH} is passed in from Bazel
    File "/oname=Moosync.exe" "${BINARY_PATH}"
    
    ; Create Shortcuts
    CreateDirectory "$SMPROGRAMS\Moosync"
    CreateShortcut "$SMPROGRAMS\Moosync\Moosync.lnk" "$INSTDIR\Moosync.exe"
    
    ; Write Uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\Moosync.exe"
    Delete "$INSTDIR\Uninstall.exe"
    Delete "$SMPROGRAMS\Moosync\Moosync.lnk"
    RMDir "$SMPROGRAMS\Moosync"
    RMDir "$INSTDIR"
SectionEnd