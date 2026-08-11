# 호환성

하드웨어와 게임 호환성은 아직 검증되지 않았습니다. 다음 표는 구현 계획과 벤치 및 실제 하드웨어 검증 상태를 구분합니다.

| 장치 또는 환경 | 지원 계획 | 검증 상태 |
|---|---:|---|
| Original Launchpad / Mk1 | MVP | 미검증 (`Unverified`) |
| Launchpad S | MVP 대체 대상 | 미검증 (`Unverified`) |
| Launchpad Mini Mk1/Mk2 | MVP 대체 대상 | 미검증 (`Unverified`) |
| Launchpad Mk2 | MVP | 미검증 (`Unverified`) |
| Launchpad X | 지원 예정 | 미검증 (`Unverified`) |
| Launchpad Mini Mk3 | 지원 예정 | 미검증 (`Unverified`) |
| Launchpad Pro Mk3 | 지원 예정 | 미검증 (`Unverified`) |
| Windows 키보드 출력 | MVP | QEMU Windows PE 벤치 검증 (`Bench verified`); 데스크톱/게임 포커스는 미검증 (`Unverified`) |
| PUMP IT UP RISE | 최종 대상 | 미검증, 실제 기기 테스트 필요 |
| RP2030 + FT232RL 브리지 | 향후 지원 | 미구현 |

보유한 구형 Launchpad의 정확한 모델은 아직 확인되지 않았습니다. 따라서 최초 실행 진단 기능은 위험한 모델 전용 명령을 무작정 보내지 않고 MIDI 포트 이름과 관찰된 프로토콜 동작을 보고할 수 있어야 합니다.
