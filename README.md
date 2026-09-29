<div align="center">

<img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="독한 사전 아이콘">

# 독한 사전 · Dokhan

**찾고, 읽고, 기억하는 독-한 / 한-독 전자사전**

[german.kr](https://german.kr/) 전자사전 ZIP 파일을 그대로 열어<br>
목차 · 색인 · 전문 검색 · 책갈피를 제공하는 데스크톱 · Android 앱입니다.

[![Release](https://img.shields.io/github/v/release/ironpark/dokhan?label=%EC%B5%9C%EC%8B%A0%20%EB%B2%84%EC%A0%84&color=205b80)](https://github.com/ironpark/dokhan/releases/latest)
[![Build](https://img.shields.io/github/actions/workflow/status/ironpark/dokhan/build-targets.yml?branch=main&label=build)](https://github.com/ironpark/dokhan/actions/workflows/build-targets.yml)
[![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Android-205b80)](#설치)
[![License](https://img.shields.io/badge/license-MPL--2.0-e3b64f)](LICENSE)

[**다운로드**](https://github.com/ironpark/dokhan/releases/latest) · [사전 파일 받기](#1-사전-파일-받기) · [개발](#개발)

</div>

<p align="center">
  <img src="docs/demo.gif" width="800" alt="독한 사전 사용 예시: 검색, 읽기 설정, 책갈피">
</p>

---

## 시작하기

### 1. 사전 파일 받기

> [!IMPORTANT]
> 독한 사전에는 사전 데이터가 들어 있지 않습니다.
> **[german.kr](https://german.kr/)** 상단 메뉴의 **전자사전 → 다운로드**에서 전자사전 ZIP 파일(예: v8.0 `dictionary8.zip`)을 받아 주세요.
> 사전 내용의 저작권은 german.kr에 있습니다.

### 2. 앱 설치

<a id="설치"></a>

[최신 릴리스](https://github.com/ironpark/dokhan/releases/latest)에서 플랫폼에 맞는 파일을 받습니다.

| 플랫폼 | 파일 | 참고 |
| --- | --- | --- |
| **macOS** (Apple Silicon) | `Dokhan_x.y.z_aarch64.dmg` | 처음 열 때 “확인되지 않은 개발자” 경고가 나오면 **시스템 설정 → 개인정보 보호 및 보안**에서 **그래도 열기**를 누릅니다. |
| **Windows** (x64) | `Dokhan_x.y.z_x64-setup.exe` | SmartScreen 경고가 나오면 **추가 정보 → 실행**을 누릅니다. MSI 설치 파일도 제공합니다. |
| **Android** | `app-universal-universal-release.apk` | 브라우저의 **출처를 알 수 없는 앱 설치**를 허용한 뒤 설치합니다. |

### 3. 사전 열기

앱을 실행하고 **ZIP 파일 선택**을 눌러 받은 ZIP을 고릅니다. 데스크톱에서는 창에 끌어 놓아도 됩니다.
**압축을 풀지 말고 ZIP 그대로** 선택하세요. 처음 한 번 색인을 만든 뒤에는 다음 실행부터 바로 열립니다.

## 기능

| 기능 | 설명 |
| --- | --- |
| 🔎 **전문 검색** | 표제어와 뜻풀이 전체를 검색합니다. 움라우트는 `ae`·`oe`·`ue`로도 찾을 수 있습니다. |
| 📚 **목차 · 색인** | 사전 목차와 표제어 색인(v8.0 기준 8만여 개)을 스크롤하는 대로 이어서 불러옵니다. |
| 📖 **읽기 좋은 본문** | 뜻풀이 번호를 정리하고, 약어 · 표기 기호에 설명을 붙입니다. 글자 크기 · 줄 간격 · 본문 폭을 조절할 수 있습니다. |
| ↔️ **뒤로 · 앞으로** | 본문 링크를 따라가도 이전 항목과 읽던 위치로 돌아올 수 있습니다. |
| 🔖 **책갈피** | 표제어와 문서를 폴더별로 저장합니다. |
| 🔄 **업데이트 알림** | 새 버전이 나오면 알려 줍니다. 데스크톱은 앱 안에서 바로 업데이트합니다. |

<details>
<summary><b>단축키 (데스크톱)</b></summary>

| 동작 | macOS | Windows |
| --- | --- | --- |
| 뒤로 | `⌘` `[` · 마우스 뒤로 버튼 | `Alt` `←` · 마우스 뒤로 버튼 |
| 앞으로 | `⌘` `]` · 마우스 앞으로 버튼 | `Alt` `→` · 마우스 앞으로 버튼 |

</details>

## 개발

Tauri v2 · Svelte 5 · TypeScript · Rust로 만들었습니다. CHM/LZX 파서는 순수 Rust로 구현했고, 전문 검색에는 [Tantivy](https://github.com/quickwit-oss/tantivy)를 씁니다.

Node.js 22.12 이상, pnpm 12, Rust stable이 필요합니다.

```bash
pnpm install
pnpm tauri dev
```

```bash
pnpm check        # 타입 검사
pnpm test         # 프론트엔드 테스트
cargo test --manifest-path src-tauri/Cargo.toml
```

실제 사전 ZIP으로 전체 파싱을 확인하려면:

```bash
DOKHAN_TEST_ZIP=/path/to/dictionary8.zip \
  cargo test --manifest-path src-tauri/Cargo.toml configured_zip_runtime_smoke
```

<details>
<summary><b>구조</b></summary>

- ZIP을 메모리에 올린 뒤 내부 CHM을 직접 파싱합니다. 반복적인 파일 시스템 접근이 없습니다.
- `master.hhc`로 목차를, `.hhk`로 색인을 만듭니다.
- 본문의 링크와 이미지(`href` / `src`)를 CHM 안에서 찾아 연결합니다.
- 사전 구축은 백그라운드에서 멀티스레드로 진행되고, 프론트엔드는 진행률을 폴링합니다.
- 성능 측정 방법과 결과는 [docs/CHM_BENCH.md](docs/CHM_BENCH.md)에 있습니다.

</details>

<details>
<summary><b>Tauri 명령</b></summary>

- `prepare_zip_source(path)`
- `start_master_build(zipPath?)`
- `get_master_build_status(zipPath?, buildKey?)`
- `get_master_contents(zipPath?)`
- `get_index_entries(prefix?, limit?, offset?, zipPath?)`
- `search_entries(query, limit?, zipPath?)`
- `get_entry_detail(id, zipPath?)`
- `get_content_page(local, sourcePath?, zipPath?)`
- `resolve_link_target(href, currentSourcePath?, currentLocal?, zipPath?)`
- `resolve_media_data_url(href, currentSourcePath?, currentLocal?, zipPath?)`

</details>

<details>
<summary><b>릴리스</b></summary>

1. `src-tauri/tauri.conf.json`, `package.json`, `src-tauri/Cargo.toml`의 버전을 올립니다.
2. `vX.Y.Z` 태그를 푸시하면 `release.yml`이 빌드하고 GitHub 릴리스를 만듭니다. 태그와 앱 버전이 다르면 빌드가 실패합니다.
3. 데스크톱 업데이트 파일은 `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` Secrets로 서명되며, 앱은 최신 릴리스의 `latest.json`을 확인합니다.

`main`에 푸시하면 `build-targets.yml`이 nightly 프리릴리스를 만듭니다. nightly는 업데이트 알림 대상이 아닙니다.

앱 아이콘 원본은 `src-tauri/icons/dokhan-icon.svg`이고, `pnpm tauri icon src-tauri/icons/icon-manifest.json`으로 플랫폼별 아이콘을 다시 만듭니다.

</details>

## 라이선스

앱 소스 코드는 [MPL-2.0](LICENSE)을 따릅니다. 사전 데이터는 이 저장소에 포함되어 있지 않으며, 저작권은 [german.kr](https://german.kr/)에 있습니다.
