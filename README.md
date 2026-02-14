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
- 권장 포맷: `SQLite + FTS5` 또는 `tantivy/meilisearch` 계열
- 오프라인 앱 공통 사용을 위해 단일 파일 DB 권장

## 7) 권장 데이터 스키마(초안)

```sql
-- 표제어/문서 메타
CREATE TABLE entry (
  id INTEGER PRIMARY KEY,
  headword TEXT NOT NULL,
  headword_norm TEXT NOT NULL,
  lang_from TEXT,         -- de/ko/mixed
  lang_to TEXT,           -- de/ko/mixed
  definition_html TEXT,   -- 원문 HTML
  definition_text TEXT,   -- 평문 변환
  source_chm TEXT NOT NULL,
  source_path TEXT NOT NULL, -- CHM 내부 htm 경로
  created_at TEXT NOT NULL
);

CREATE INDEX idx_entry_headword_norm ON entry(headword_norm);

-- 동의어/변형어
CREATE TABLE entry_alias (
  entry_id INTEGER NOT NULL,
  alias TEXT NOT NULL,
  alias_norm TEXT NOT NULL,
  FOREIGN KEY(entry_id) REFERENCES entry(id)
);
CREATE INDEX idx_entry_alias_norm ON entry_alias(alias_norm);

-- FTS(예: SQLite FTS5)
-- CREATE VIRTUAL TABLE entry_fts USING fts5(headword, definition_text, content='entry', content_rowid='id');
```

## 8) Tauri 단일 스택 전략 (모바일 + 데스크탑)

- 앱 프레임워크: `Tauri v2 + Svelte + TypeScript`
- 공통 코어: Tauri Rust 명령(`#[tauri::command]`)으로 ZIP/CHM 파싱, 인덱싱, 검색 처리
- UI 계층:
  - 데스크탑(Windows/macOS): 동일 Svelte UI + 동일 Rust 코어
  - 모바일(iOS/Android): Tauri 모바일 타겟 + 동일 Svelte UI + 동일 Rust 코어
- 데이터 저장:
  - 권장 `SQLite(FTS5)` 단일 파일 DB를 앱 로컬 저장소에 생성/갱신
  - 초기 배포는 원본 ZIP 포함 또는 첫 실행 시 ZIP 선택 후 인덱스 생성
- 이점:
  - 플랫폼별 검색 로직 중복 제거
  - 파서/인덱서 품질을 한 코드베이스에서 유지

## 9) 구현 체크리스트

- [x] ZIP 리더에서 `.chm` 엔트리 목록 생성
- [x] CHM 바이너리 기반 `*.htm/*.hhk/*.hhc` 경로 프리뷰 명령 추가(초기 파싱 단계)
- [ ] CHM 파서 라이브러리 선정 및 샘플 3개(`master`, `merge01`, `merge36`) 정밀 검증
- [ ] `*.hhk` 기반 표제어 추출 정확도 테스트
- [ ] `*.htm` 본문 정규화/언어 판별 규칙 확정
- [ ] SQLite 인덱서/검색 API 구현
- [ ] 모바일/데스크탑 공용 검색 UX(자동완성/하이라이트/히스토리) 연결

## 10) 현재 구현된 Tauri 명령

- `analyze_default_dataset()`: 기본 ZIP(`asset/dictionary_v77.zip`) 통계 분석
- `analyze_zip_dataset(zipPath)`: 지정 ZIP 통계 분석
- `preview_chm_paths(zipPath?, sampleLimit?)`: CHM별 HTML/HHK/HHC 경로 프리뷰
- `extract_headwords_preview(zipPath?, chmFile?, sampleLimit?)`: CHM 내부 HTML 경로명 기반 표제어 샘플 추출(초기 버전)
- `extract_headwords_from_hhk(zipPath?, chmFile?, sampleLimit?)`: CHM 바이너리 내 HHK `param name="Name"` 패턴 기반 표제어 샘플 추출(1차 구현)
- `validate_dataset_pipeline(zipPath?)`: 전체 CHM을 순회하며 커버리지/표제어 추정치/경고를 생성하는 실행 검증 리포트
- `build_master_features(debugRoot?)`: `master.hhc` + `merge*.htm`를 해석해 내용/색인/검색용 런타임 인덱스 로드
- `get_master_contents(debugRoot?)`: 내용 탭 데이터(목차)
- `get_index_entries(prefix?, limit?, debugRoot?)`: 색인 탭 데이터(접두어 조회)
- `search_entries(query, limit?, debugRoot?)`: 검색 탭 데이터(본문 검색)
- `get_entry_detail(id, debugRoot?)`: 본문 상세 조회

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
