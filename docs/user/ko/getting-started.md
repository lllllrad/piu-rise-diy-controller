# 시작하기 및 실제 게임 테스트

현재 빌드는 엔지니어링 MVP입니다. 구형 Launchpad를 안전하게 식별할 수
있도록 모델 선택 SysEx나 LED 명령을 자동 전송하지 않으며, 먼저 MIDI
입력을 수동 관찰합니다.

## 브라우저 레이아웃 편집기 미리보기

최신 브라우저에서 `web/index.html`을 직접 엽니다. 설치 없이 Tauri 호스트가
포함하는 것과 같은 Mk2 장치 두 대의 surface 편집기를 확인할 수 있습니다.
브라우저 모드는 허용된 단일·복수 패널 할당을 검증하고 데모 레이아웃을
브라우저 로컬 저장소에 보존합니다. MIDI, LED 또는 키보드 출력에는 접근하지
않습니다.

Tauri 호스트는 `src-tauri/`에 있습니다. Rust에서 레이아웃을 다시 검증하고
버전이 있는 JSON을 애플리케이션 설정 디렉터리에 저장합니다. 저장된
레이아웃을 실행 중인 실제 컨트롤러에 연결하는 기능은 아직 구현되지 않았으며
`Unverified`입니다.

## 1. Windows에서 빌드

Windows SDK를 포함한 Visual Studio 2022 C++ Build Tools를 설치한 다음 Rust
1.97.1을 직접 설치하거나 `mise install`을 실행합니다. 일반 사용자 권한 개발
셸에서 빌드합니다.

```powershell
cargo build --release --locked
```

실행 파일은 `target\release\piu-rise-controller.exe`입니다. 포함된
매니페스트가 관리자 권한을 요청합니다.

## 2. MIDI 포트와 프로토콜 계열 식별

Launchpad 한 대를 연결한 후 실행합니다.

```powershell
piu-rise-controller list
piu-rise-controller -vv monitor --input "Launchpad"
```

왼쪽 아래 그리드 패드, 위쪽 버튼, 오른쪽 버튼을 각각 눌러 출력을
저장합니다. 일반적인 프로토콜 계열은 다음과 같습니다.

- `original`: 왼쪽 위 그리드 패드가 `0`이고 행 간격이 16인 Original/Mk1
  주소 `0..119`
- `launchpad-s`: Original/Mk1 주소 계열
- `mini-legacy`: Mini Mk1/Mk2 레거시 주소 계열
- `mk2`: `11..88` 형태의 RGB Launchpad Mk2 그리드 주소

관찰한 주소가 다르면 실제 키 출력을 실행하지 마십시오. 설정 파일을
생성한 다음 monitor 결과에 맞춰 `bindings`를 수정합니다.

구형 Launchpad는 문서화된 그리드 주소를 출력하기 전에 장치에서 User 또는
standalone 레이아웃을 선택해야 할 수 있습니다. 정확한 모델을 확인하기
전까지 애플리케이션은 해당 모드를 자동 변경하지 않습니다. 재연결할 때마다
`monitor`로 모드를 확인합니다.

## 3. 설정 생성 및 수정

다음 중 하나를 선택합니다.

```powershell
piu-rise-controller write-default-config --model original
piu-rise-controller write-default-config --model mk2 --force
piu-rise-controller write-default-config --model mk2 --two-devices --force
```

`piu-rise-controller doctor`로 정확한 설정 및 로그 경로를 확인합니다.
`device.input_port`를 MIDI 입력 포트 하나만 식별하는 값으로 수정하고,
Windows 출력을 활성화하기 전에 모든 키와 binding을 검토합니다.
장치 두 대를 쓰면 `device.input_port_right`도 설정합니다. `device = 0` binding은
왼쪽, `device = 1` binding은 오른쪽 장치입니다. P2 예제 키 `Z X C V B`는
RISE 키 설정에서도 동일하게 연결해야 합니다.

## 4. 전체 매핑 dry-run

### Launchpad Mk2 5키 배치

입력 한 대와 `--model mk2`를 사용하면 물리적인 8×8 그리드의 맨 위 행을 사용하지 않습니다.
그보다 위에 있는 원형 버튼 8개를 왼쪽부터 `W`, `S`, `A`, `D`, `Enter`, `Esc`, `Space`,
`Tab`에 사용합니다.
Mk2 오른쪽 측면 버튼은 사용하지 않습니다. 6K 또는 10K에서 장치 두 대를 사용하면 정상 방향인
오른쪽 메인 장치의 상단 버튼이 기본 UI 키를 담당합니다. 왼쪽 장치는 반시계 방향으로 90도
회전하며, 회전 후 물리적으로 왼쪽에 오는 원래 상단 버튼을 `Q`, `E`, `F1`, `F2`, `F3`,
`F5`, `F6`, `F7`에 사용합니다. 나머지 7개 행에는
빨간색 3×3 위쪽 발판 두 개, 파란색 3×3 아래쪽 발판 두 개와 노란색 세로 3행×가로 4열
중앙 발판을 배치합니다. 위쪽 겹침은 어두운 빨간색, 아래쪽 겹침은 어두운 파란색입니다.
겹침 셀은 해당 방향 발판과 중앙 발판을 동시에 누르고 동시에 뗍니다. 실제 출력 모드가 시작되면
애플리케이션이 Mk2 MIDI 출력 포트를 열고 이 배치를 밝힙니다. 입력 포트 문자열로 출력 포트를
고를 수 없으면 `device.output_port`를 지정하십시오. 정상적인 Ctrl+C 종료 시 LED를 끕니다.

이 MIDI 주소와 팔레트 동작은 소유자의 Launchpad Mk2에서 확인하기 전까지 `Unverified`입니다.
`--dry-run`은 의도적으로 LED 출력을 보내지 않습니다.

### 장치 두 대를 사용하는 10패널 배치

정상 방향인 오른쪽 메인 장치는 `--input`/`--input-index`, 반시계 방향으로 회전한 추가
장치는 `--input-left`/`--input-left-index`로 지정합니다. 왼쪽 입력을 지정하면 항상
10패널 배치를 사용합니다. RISE 6K에서도 열 개 출력 키 설정을 필요에 맞게 수정해 이 배치를
그대로 사용할 수 있습니다. 왼쪽 장치는 P1 5패널 전체를, 오른쪽 장치는 P2 5패널 전체를
사용합니다. 물리적으로 반시계 방향인 왼쪽 장치의 입력과 LED 주소에는 시계 방향 보정 변환을
적용합니다.

동일 장치의 포트 이름이 같으면 `list`가 보여 주는 번호를 사용합니다. LED 출력도
메인 출력은 `--output-index`, 왼쪽 출력은 `--output-left-index`로 선택할 수 있습니다. USB를 다시 연결하면 번호가
바뀔 수 있으므로 실행 전에 `list`를 다시 확인하십시오.

dry-run은 MIDI 입력과 press/reference/release 상태를 모두 처리하지만
Windows 키는 전송하지 않습니다.

```powershell
piu-rise-controller -vv run --input "Launchpad" --model original --dry-run
```

짧게 누르기, 길게 누르기, 같은 논리 패널에 속한 패드 두 개, 동시 입력,
Ctrl+C 종료를 확인합니다. 로그에서 모든 press에 대응하는 release가 있어야
합니다.

## 5. 게임 밖에서 Windows 출력 테스트

신뢰할 수 있는 키 이벤트 확인 프로그램이나 텍스트 편집기를 연 다음
애플리케이션을 관리자 권한으로 실행합니다.

```powershell
piu-rise-controller output-test --key F --hold-ms 100
piu-rise-controller run --input "Launchpad" --model original
```

콘솔에 접근할 수 있는 상태를 유지합니다. Ctrl+C는 종료 전에 Release All을
요청합니다. 패드를 누른 상태에서 작업 관리자로 프로세스를 강제 종료하지
마십시오.

이전 실행이 비정상 종료됐다면 관리자 권한 콘솔에서 다음을 실행합니다.

```powershell
piu-rise-controller release-all
```

## 6. PUMP IT UP RISE 연동 테스트

위험이 낮은 메뉴와 낮은 난이도 채보부터 시작합니다.

1. `doctor`가 `elevated=true`를 출력하는지 확인합니다.
2. 게임 플레이 전에 메뉴 키와 `Esc` 위치를 확인합니다.
3. 단일 입력, 롱 노트, 동시 롱 노트 순서로 확인합니다.
4. Ctrl+C 후 게임 키가 눌린 채 남지 않았는지 확인합니다.
5. 테스트에 사용한 정확한 설정과 로그를 보관합니다.

실제 RISE 환경에서 이 과정을 수행하고 실행 파일 버전과 장치 모델을
기록하기 전까지 결과는 `Unverified`입니다.
