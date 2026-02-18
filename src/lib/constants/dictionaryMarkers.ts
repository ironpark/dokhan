export type MarkerGroup =
  | "register"
  | "region"
  | "time"
  | "usage"
  | "domain"
  | "meaning"
  | "orthography"
  | "grammar"
  | "misc";

export type MarkerMeta = {
  group: MarkerGroup;
  tooltip: string;
};

export const ROUND_MARKER_GROUPS: Record<string, MarkerGroup> = {
  통용어: "register",
  경: "register",
  속어: "register",
  비어: "register",
  교양어: "register",
  아어: "register",
  시어: "register",
  반어: "usage",
  은어: "register",
  강조: "usage",
  과시: "usage",
  "은폐/미화": "usage",
  친근: "usage",
  농: "usage",
  폄: "usage",
  욕: "usage",
  지역적: "region",
  "südd., österr.": "region",
  "schweiz.": "region",
  "nordd.": "region",
  berlin: "region",
  역사적: "time",
  준고어: "time",
  고어: "time",
  옛: "time",
  구제: "time",
  전에는: "time",
  드물게: "usage",
  또한: "usage",
  "부정어와 함께": "usage",
  "기쁨, 경탄, 동경 등에 쓰임": "usage",
};

export const ROUND_MARKER_META: Record<string, MarkerMeta> = {
  통용어: {
    group: "register",
    tooltip:
      "정상어에 가깝고 일상적·친숙한 상황에서 통상적으로 쓰이는 표현 (ugs. umgangssprachlich)",
  },
  경: {
    group: "register",
    tooltip: "경솔하거나 거친 태도가 드러나는 표현 (salopp)",
  },
  속어: {
    group: "register",
    tooltip: "거칠고 천한 뉘앙스의 표현 (derb)",
  },
  비어: {
    group: "register",
    tooltip: "외설적/야비한 표현 (vulg. vulgaer)",
  },
  교양어: {
    group: "register",
    tooltip: "상대적으로 교양층 문체에 속하는 표현 (bildungssprachlich)",
  },
  아어: {
    group: "register",
    tooltip: "격식/엄숙한 자리 또는 문학에서 쓰이는 표현 (gehoben)",
  },
  시어: {
    group: "register",
    tooltip: "현대 일반어에서는 드문 문학적·의고적 표현 (dichterisch)",
  },
  반어: { group: "usage", tooltip: "반어적 용법 (ironisch)" },
  은어: { group: "register", tooltip: "은어/특정 집단의 전문적 비공식 표현 (Jargon)" },
  강조: { group: "usage", tooltip: "강조 목적의 표기" },
  과시: { group: "usage", tooltip: "과시적 뉘앙스의 용법" },
  "은폐/미화": { group: "usage", tooltip: "완곡·미화·은폐 목적의 용법 (verhuellend)" },
  친근: { group: "usage", tooltip: "친근한 말투의 용법 (familiär)" },
  농: { group: "usage", tooltip: "농담조의 용법 (scherzhaft)" },
  폄: { group: "usage", tooltip: "폄하/경멸적 뉘앙스의 표현 (abwertend)" },
  욕: { group: "usage", tooltip: "욕설/모욕적 표현 (schimpf)" },
  지역적: {
    group: "region",
    tooltip: "특정 지역에서 주로 쓰이나 경계가 엄밀히 고정되진 않은 지역어 (landsch.)",
  },
  "südd., österr.": { group: "region", tooltip: "남독일/오스트리아 지역 용법" },
  "schweiz.": { group: "region", tooltip: "스위스 지역 용법" },
  "nordd.": { group: "region", tooltip: "북독일 지역 용법" },
  berlin: { group: "region", tooltip: "베를린 지역 용법" },
  역사적: { group: "time", tooltip: "역사적 사태/시대에 한정되는 표현 (historisch)" },
  준고어: { group: "time", tooltip: "점차 사용이 줄어드는 준고어 표현 (veraltend)" },
  고어: { group: "time", tooltip: "현대 일반어에서는 사어에 가까운 표현 (veraltet)" },
  옛: { group: "time", tooltip: "과거 용법/예전 사태를 가리키는 표현 (frueher)" },
  구제: { group: "time", tooltip: "과거 제도·구식 체계 관련 표현 (frueher)" },
  전에는: { group: "time", tooltip: "예전 용법/과거 상태를 지시 (frueher)" },
  드물게: { group: "usage", tooltip: "사용 빈도가 낮은 용법" },
  또한: { group: "usage", tooltip: "추가·대안 용법 표기" },
  "부정어와 함께": { group: "usage", tooltip: "부정어와 결합해 주로 쓰이는 용법" },
  "기쁨, 경탄, 동경 등에 쓰임": {
    group: "usage",
    tooltip: "감정(기쁨/경탄/동경 등) 표현 맥락에서 쓰이는 용법",
  },
};

export const SQUARE_MARKER_GROUPS: Record<string, MarkerGroup> = {
  가: "domain",
  관: "domain",
  광: "domain",
  군: "domain",
  법: "domain",
  상: "domain",
  핵: "domain",
  심리: "domain",
  사회: "domain",
  물리: "domain",
  고고: "domain",
  전의: "meaning",
  성구: "usage",
  속담: "usage",
  격언: "usage",
};

export const SQUARE_MARKER_META: Record<string, MarkerMeta> = {
  가: { group: "domain", tooltip: "가톨릭 관련 용어" },
  관: { group: "domain", tooltip: "관청어/행정 실무 용어" },
  광: { group: "domain", tooltip: "광업/광부 관련 용어" },
  군: { group: "domain", tooltip: "군사 용어" },
  법: { group: "domain", tooltip: "법률 용어" },
  상: { group: "domain", tooltip: "상업/상인 관련 용어" },
  핵: { group: "domain", tooltip: "핵공학 관련 용어" },
  심리: { group: "domain", tooltip: "심리학 관련 용어" },
  사회: { group: "domain", tooltip: "사회학 관련 용어" },
  물리: { group: "domain", tooltip: "물리학 관련 용어" },
  고고: { group: "domain", tooltip: "고고학 관련 용어" },
  전의: { group: "meaning", tooltip: "본래 의미에서 확장/전이된 뜻(비유 등)" },
  성구: { group: "usage", tooltip: "관용구/성구 표기" },
  속담: { group: "usage", tooltip: "속담 용법" },
  격언: { group: "usage", tooltip: "격언 용법" },
};

export const ANGLE_MARKER_ALIASES: Record<string, string> = {
  단수만: "복수 없음",
};

export const ANGLE_MARKER_META: Record<string, string> = {
  "복수 없음": "문법 수(數) 정보: 복수형이 일반적으로 쓰이지 않음",
  "대개 단수": "문법 수(數) 정보: 보통 단수로 쓰임",
  "Adj.": "품사 표시: 형용사",
  "Adv.": "품사 표시: 부사",
  Verb: "품사 표시: 동사",
  "Subst.": "품사 표시: 명사",
};
