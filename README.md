# Dokhan

Tauri(v2) + Svelte + TypeScript + Rust 기반 독-한/한-독 전자사전 앱입니다.
입력은 `asset/dictionary_v77.zip` 같은 ZIP 파일 1개이며, 내부 CHM을 직접 파싱해 내용/색인/검색을 제공합니다.

## 주요 특징

- ZIP 파일을 메모리에 로드한 뒤 처리 (반복 파일시스템 접근 최소화)
- 순수 Rust CHM/LZX 파서 통합
- `master.hhc` 기반 목차, `.hhk` 기반 실제 색인
- 본문 링크/이미지(`href`/`src`) 해석 지원
- 비동기 빌드 + 진행률 폴링
- 멀티스레드 파싱 (플랫폼별 스레드 제한 적용)

## 기술 스택

- Frontend: Svelte 5, TypeScript, Bun
- Desktop/Mobile: Tauri v2
- Backend: Rust
- Search: Tantivy

## 실행

```bash
bun install
bun tauri dev
```

## 검증

```bash
bun check
cargo test --manifest-path src-tauri/Cargo.toml
```

## Tauri Command API

- `prepare_zip_source(path)`
- `start_master_build(zipPath?)`
- `get_master_build_status(zipPath?)`
- `get_master_contents(zipPath?)`
- `get_index_entries(prefix?, limit?, zipPath?)`
- `search_entries(query, limit?, zipPath?)`
- `get_entry_detail(id, zipPath?)`
- `get_content_page(local, sourcePath?, zipPath?)`
- `resolve_link_target(href, currentSourcePath?, currentLocal?, zipPath?)`
- `resolve_media_data_url(href, currentSourcePath?, currentLocal?, zipPath?)`

## 데이터셋

- 기본 테스트 데이터: `asset/dictionary_v77.zip`
- 예시 통계: `chm 120`, `lnk 2`, `txt 1`

## 식별자 / 이름

- Product Name: `Dokhan`
- Tauri Identifier: `io.github.ironpark.dokhan`

## 라이선스

- `MPL-2.0` (파일: `LICENSE`)
