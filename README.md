# german.kr

Tauri(v2) + Svelte + TypeScript + Rust 기반 독-한/한-독 전자사전 뷰어입니다.  
입력은 `asset/dictionary_v77.zip` 같은 ZIP 파일 1개이며, 내부 CHM들을 직접 읽어 내용/색인/검색을 제공합니다.

## 기본 구현

- ZIP 단일 입력만 사용합니다.
- 외부 도구/의존성 없이 Rust 코드에서 ZIP/CHM을 직접 처리합니다.
- 인덱스는 메모리 런타임으로 구성합니다

## 현재 구현 범위

- CHM 컨테이너 파싱(디렉터리/섹션/스트림)
- LZX 압축 해제(Rust 구현)
- `master.hhc` 기반 목차(Content)
- CHM 실제 `.hhk` 기반 색인(Index)
- 본문 텍스트 검색(Search)
- 본문 내부 링크/이미지(`href`/`src`) 해석 및 렌더링 지원
- 로딩 진행률 상태 폴링 및 UI 반영

## 프로젝트 구조

```text
src-tauri/src
├─ app
│  ├─ commands.rs   # tauri command entrypoint
│  └─ model.rs      # 공용 DTO/응답 모델
├─ chm
│  ├─ archive.rs    # CHM archive reader
│  ├─ lzx.rs        # LZX decoder
│  └─ archive/
│     ├─ headers.rs
│     ├─ directory.rs
│     └─ compression.rs
├─ parsing
│  ├─ dataset.rs    # ZIP 스캔/요약
│  ├─ index.rs      # hhk/headword 파싱
│  └─ text.rs       # html 텍스트 정규화
├─ runtime
│  ├─ state.rs      # 빌드 상태/런타임 로딩
│  ├─ search.rs     # 색인/검색 조회
│  ├─ link_media.rs # 링크/미디어 해석
│  └─ zip.rs        # zip -> runtime materialize
└─ lib.rs           # backend crate entry
```

## 개발 실행

```bash
bun install
bun tauri dev
```

## 검증 명령

```bash
bun check
cargo test --manifest-path src-tauri/Cargo.toml
```

## 프론트 동작

- 초기 화면에서 ZIP 파일을 드래그 앤 드롭하면 빌드를 시작합니다.
- 빌드 완료 후 `내용 / 색인 / 검색` 탭을 사용할 수 있습니다.
- 색인은 입력 즉시(prefix) 필터링됩니다.

## Tauri Command API

- `analyze_zip_dataset(zipPath)`
- `start_master_build(zipPath?)`
- `get_master_build_status(zipPath?)`
- `get_master_contents(zipPath?)`
- `get_index_entries(prefix?, limit?, zipPath?)`
- `search_entries(query, limit?, zipPath?)`
- `get_entry_detail(id, zipPath?)`
- `get_content_page(local, sourcePath?, zipPath?)`
- `resolve_link_target(href, currentSourcePath?, currentLocal?, zipPath?)`
- `resolve_media_data_url(href, currentSourcePath?, currentLocal?, zipPath?)`

## 데이터셋 참고

- `asset/dictionary_v77.zip`
- ZIP 내부 통계(기준 데이터셋): `chm 120`, `lnk 2`, `txt 1`

## 주의 사항

- `zipPath`는 실제 존재하는 ZIP 경로여야 합니다.
- 상대 경로 실행 위치에 따라 `asset/dictionary_v77.zip` 해석이 달라질 수 있으므로, 필요하면 절대 경로를 사용하세요.
