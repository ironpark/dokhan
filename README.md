# german.kr 전자사전 데이터셋 분석 및 파싱 가이드

## 1) 목적
`asset` 폴더의 CHM 기반 전자사전 파일 구성을 정리하고,
향후 **모바일 + 데스크탑(Windows/macOS)** 에서 동작하는 사전 앱을 위해
**ZIP 파일 입력으로 인덱스를 구성하는 파서/빌더 설계 기준**을 문서화한다.

## 2) 현재 파일 구성

### 루트
- `asset/dictionary_v77.zip`: 배포용 압축본
- `asset/dictionary_v77/`: 압축 해제본(동일 데이터)

### 확장자별 개수
- `.chm`: 120개
- `.lnk`: 2개
- `.txt`: 1개 (`readme.txt`)

### 용량(로컬 측정)
- `asset/dictionary_v77.zip`: 약 26MB
- `asset/dictionary_v77/`: 약 28MB
- ZIP 내부 파일 총 원본 크기: `28,867,394 bytes`
- ZIP 내부 파일 총 압축 크기: `26,798,115 bytes`

## 3) CHM 파일 네이밍 패턴

### 관찰된 분류
- 메타/엔트리: `master.chm` (1개)
- 본권: `merge01.chm`, `merge02.chm`, ..., `merge36.chm` 중 **35개 존재**
- 세부 분권: `merge01-01.chm`, `merge11-02-03.chm` 같은 하위 파트 **82개**
- 증보: `merge_suppl_1.chm`, `merge_suppl_2.chm` (2개)

### 특이사항
- `merge03.chm`는 없음(대신 `merge03-01`~`merge03-05` 존재)
- 윈도우 바로가기(`.lnk`) 2개는 앱 파싱 대상에서 제외 가능

## 4) CHM 내부에서 확인한 공통 구조(샘플 추출)

CHM 문자열 검사 기준으로 다음 구조가 반복적으로 보인다.

- 인덱스/토픽 계열
  - `/#TOPICS`
  - `/$WWKeywordLinks/BTree`
  - `/$WWKeywordLinks/Data`
  - `/$WWKeywordLinks/Map`
- 문서/목차/인덱스 파일
  - `*.htm`, `*.html`
  - `*.hhc` (목차)
  - `*.hhk` (키워드 인덱스)

예시:
- `master.chm` 내부: `master.hhc`, `master.hhk`, `master.html`, `version_information.htm`
- `merge01.chm` 내부: `/Aal.htm`, `/ab.htm`, `/abbauen.htm` 등 표제어 페이지

즉, 각 CHM은 HTML Help 표준 컨테이너이며,
실제 사전 본문은 `*.htm` 문서 집합으로 구성되어 있을 가능성이 높다.

## 5) 제공 `readme.txt` 요약

`asset/dictionary_v77/readme.txt`는 CP949 인코딩 한국어 문서이며,
버전 히스토리/사용법/저작권 안내를 포함한다.

핵심 내용:
- 독-한/한-독 전자사전
- 현재 공개 버전 7.7 (문서상 2025-08-15 공개 표기)
- 상업적 이용 금지(원문 저작권 문구 참조)

## 6) ZIP 입력 기반 인덱싱 프로그램 요구사항(권장)

### 입력
- 단일 ZIP 파일 경로
- ZIP 내부에 CHM 다수 포함(`master.chm`, `merge*.chm`)

### 처리 파이프라인
1. ZIP 스캔
- 엔트리 목록 수집
- `.chm`만 필터링
- 파일명 기준 정렬(자연 정렬 권장)

2. CHM 추출(메모리 or 임시 디렉터리)
- ZIP 엔트리를 스트림으로 읽고 CHM 파서에 전달
- 대용량 대비해 순차 처리(동시성 제한)

3. CHM 파싱
- 우선순위: `.hhk`(키워드), `.hhc`(목차), `*.htm*`(본문)
- 표제어/동의어/본문 텍스트/원본 경로 추출

4. 정규화
- 유니코드 정규화(`NFC` 권장)
- 검색용 토큰 필드 생성
  - 독일어: 소문자/움라우트 처리 규칙 정의
  - 한국어: 공백/기호 정리 + 초성 검색 여부 정책화

5. 인덱스 빌드
- Prefix 인덱스(자동완성)
- Full-text 인덱스(본문 검색)
- Exact 키 인덱스(표제어 직검색)

6. 저장
- 현재: 앱 런타임 메모리 인덱스(영속 저장소 미사용)
- 향후: 온디스크 캐시/검색엔진은 별도 결정(요구사항 확정 후)

## 7) 런타임 데이터 모델(현재)

- `content tree`: `master.hhc` 기반 목차 노드 트리
- `index entries`: CHM 실제 색인(`.hhk`) 기반 엔트리 목록
- `search index`: 본문 텍스트 기반 메모리 검색 인덱스
- `entry detail`: 원본 HTML + 텍스트 + 소스(`chm/path`) 메타

## 8) Tauri 단일 스택 전략 (모바일 + 데스크탑)

- 앱 프레임워크: `Tauri v2 + Svelte + TypeScript`
- 공통 코어: Tauri Rust 명령(`#[tauri::command]`)으로 ZIP/CHM 파싱, 인덱싱, 검색 처리
- UI 계층:
  - 데스크탑(Windows/macOS): 동일 Svelte UI + 동일 Rust 코어
  - 모바일(iOS/Android): Tauri 모바일 타겟 + 동일 Svelte UI + 동일 Rust 코어
- 데이터 저장:
  - 현재는 ZIP 로드 시 메모리 인덱스를 구성해 즉시 조회
  - 영속 저장은 미적용(추후 별도 스토리지 전략 도입 예정)
- 이점:
  - 플랫폼별 검색 로직 중복 제거
  - 파서/인덱서 품질을 한 코드베이스에서 유지

## 9) 구현 체크리스트

- [x] ZIP 리더에서 `.chm` 엔트리 목록 생성
- [x] CHM 바이너리 기반 `*.htm/*.hhk/*.hhc` 경로 프리뷰 명령 추가(초기 파싱 단계)
- [ ] CHM 파서 라이브러리 선정 및 샘플 3개(`master`, `merge01`, `merge36`) 정밀 검증
- [ ] `*.hhk` 기반 표제어 추출 정확도 테스트
- [ ] `*.htm` 본문 정규화/언어 판별 규칙 확정
- [ ] 영속 스토리지 전략 확정 및 통합(선택 사항)
- [ ] 모바일/데스크탑 공용 검색 UX(자동완성/하이라이트/히스토리) 연결

## 10) 현재 구현된 Tauri 명령

- `analyze_zip_dataset(zipPath)`: 지정 ZIP 통계 분석
- `start_master_build(zipPath?)`: 런타임 인덱스 비동기 빌드 시작
- `get_master_build_status(zipPath?)`: 빌드 진행/완료 상태 조회
- `get_master_contents(zipPath?)`: 내용 탭 데이터(목차)
- `get_index_entries(prefix?, limit?, zipPath?)`: 색인 탭 데이터(접두어 조회)
- `search_entries(query, limit?, zipPath?)`: 검색 탭 데이터(본문 검색)
- `get_entry_detail(id, zipPath?)`: 본문 상세 조회
- `get_content_page(local, sourcePath?, zipPath?)`: 목차/링크 기반 본문 페이지 조회
- `resolve_link_target(href, currentSourcePath?, currentLocal?, zipPath?)`: 내부 링크 대상 해석
- `resolve_media_data_url(href, currentSourcePath?, currentLocal?, zipPath?)`: 이미지 등 미디어를 data URL로 변환

## 11) 참고 경로
- `asset/dictionary_v77.zip`
- `asset/dictionary_v77/readme.txt`
- `asset/dictionary_v77/master.chm`
- `asset/dictionary_v77/merge01.chm`
- `asset/dictionary_v77/merge36.chm`

## 12) 테스트
- Rust 단위테스트:
  - ASCII run 추출
  - CHM 경로(`htm/hhk/hhc`) 추출
  - 분할본(`mergeNN-*`) 커버리지 판정
  - 실제 데이터셋 스모크 검증(`dictionary_v77.zip` 존재 시 CHM/LNK/TXT 개수 및 커버리지 확인)
