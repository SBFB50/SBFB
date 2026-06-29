#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 80 (front rapid-add) - i18n gate (6): no POSITIVE-VERDICT word in ANY
# locale's translation VALUES. Cross-locale twin of scan-front-discipline.sh.
#
# The cardinal invariant - the Operator RESTITUTES a verdict, never asserts one
# - leaks through translation: scan-front-discipline.sh scans source literals,
# while this gate scans the msgstr VALUES of every src/i18n/locales/*.po.
#
# Matching is per-locale and explicit:
#   - boundary: Unicode letter/mark boundaries for space-delimited scripts.
#   - substring: unspaced scripts, with locale-specific leading negator skips.
#
# A locale without a word-list FAILS loudly rather than silently degrading to a
# universal PASS-only scan. Scans msgstr only (skips the header, # comments, and
# obsolete #~ lines).
set -euo pipefail
cd "$(dirname "$0")/.."

DIR="src/i18n/locales"
if [ ! -d "$DIR" ] || [ -z "$(ls -1 "$DIR"/*.po 2>/dev/null || true)" ]; then
  echo "check-i18n-verdict-cross-locale: FAILED (no .po catalogs found under $DIR)"
  exit 1
fi

node --input-type=module <<'NODE'
import { readFileSync, readdirSync } from 'node:fs'

const DIR = 'src/i18n/locales'
const PREFIX = 'check-i18n-verdict-cross-locale'

// Universal SCREAMING-case Latin badges - language-agnostic (a pasted English
// "APPROVED" in any locale's msgstr is a verdict). ASCII, case-sensitive.
const UNIVERSAL = ["PASS","PASSED","APPROVED","VERIFIED","VALIDATED","SUCCEEDED","SUCCESS"]

// Curated per-locale positive-verdict words. The neutral status labels the
// front DOES render (tenue/met/cumplida/etc.) are deliberately
// absent, so they never trip.
const PER_LOCALE = {
  "fr": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Réussi",
      "Réussie",
      "Réussite",
      "Validé",
      "Validée",
      "Vérifié",
      "Vérifiée",
      "Verifie",
      "Verifiee",
      "Approuvé",
      "Approuvée",
      "Approuve"
    ]
  },
  "en": {
    "strategy": "boundary",
    "negators": "",
    "words": []
  },
  "es": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Aprobado",
      "Aprobada",
      "Validado",
      "Validada",
      "Verificado",
      "Verificada",
      "Superado",
      "Superada",
      "Exitoso",
      "Exitosa"
    ]
  },
  "ar": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "نجح",
      "ناجح",
      "نجاح",
      "اجتاز",
      "مقبول",
      "معتمد",
      "تم التحقق",
      "مصادق عليه",
      "موافق عليه"
    ]
  },
  "zh": {
    "strategy": "substring",
    "negators": "不未没否無毋勿別别",
    "words": [
      "通过",
      "已通过",
      "验证通过",
      "已验证",
      "批准",
      "已批准",
      "核准",
      "成功",
      "合格",
      "通過",
      "已驗證",
      "核準"
    ]
  },
  "ru": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Пройдено",
      "Одобрено",
      "Проверено",
      "Подтверждено",
      "Успешно",
      "Зачтено"
    ]
  },
  "de": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Bestanden",
      "Genehmigt",
      "Freigegeben",
      "Verifiziert",
      "Validiert",
      "Erfolgreich"
    ]
  },
  "nl": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Geslaagd",
      "Goedgekeurd",
      "Geverifieerd",
      "Gevalideerd",
      "Succesvol"
    ]
  },
  "sv": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Godkänd",
      "Bestått",
      "Verifierad",
      "Validerad",
      "Lyckad"
    ]
  },
  "pt-br": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "aprovada",
      "aprovado",
      "validada",
      "validado",
      "verificada",
      "verificado",
      "passou",
      "bem-sucedida",
      "bem-sucedido",
      "sucesso"
    ]
  },
  "it": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "superata",
      "superato",
      "approvata",
      "approvato",
      "validata",
      "validato",
      "convalidata",
      "convalidato",
      "verificata",
      "verificato",
      "riuscita",
      "riuscito",
      "promossa",
      "promosso"
    ]
  },
  "ro": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "trecut",
      "trecută",
      "aprobat",
      "aprobată",
      "validat",
      "validată",
      "verificat",
      "verificată",
      "reușit",
      "reușită"
    ]
  },
  "pl": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Zaliczone",
      "Zatwierdzone",
      "Zweryfikowane",
      "Zwalidowane",
      "Pomyślne",
      "Zaakceptowane"
    ]
  },
  "uk": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Пройдено",
      "Схвалено",
      "Перевірено",
      "Підтверджено",
      "Успішно",
      "Зараховано"
    ]
  },
  "cs": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Prošlo",
      "Schváleno",
      "Ověřeno",
      "Validováno",
      "Úspěšné",
      "Potvrzeno"
    ]
  },
  "sr-latn": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Prošlo",
      "Odobreno",
      "Provereno",
      "Verifikovano",
      "Validirano",
      "Uspešno"
    ]
  },
  "fi": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Hyväksytty",
      "Läpäisty",
      "Vahvistettu",
      "Validoitu",
      "Onnistui",
      "Todennettu"
    ]
  },
  "hu": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Jóváhagyva",
      "Megfelelt",
      "Átment",
      "Hitelesítve",
      "Validálva",
      "Sikeres"
    ]
  },
  "lt": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Patvirtinta",
      "Patikrinta",
      "Validuota",
      "Sėkminga",
      "Išlaikyta"
    ]
  },
  "el": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Πέρασε",
      "Εγκρίθηκε",
      "Επαληθεύτηκε",
      "Επικυρώθηκε",
      "Επιτυχής",
      "Επιτυχία"
    ]
  },
  "hy": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Անցավ",
      "Հաստատված",
      "Վավերացված",
      "Ստուգված",
      "Հաջողված"
    ]
  },
  "ka": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "გავიდა",
      "დამტკიცებული",
      "დადასტურებული",
      "ვალიდირებული",
      "წარმატებული"
    ]
  },
  "he": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "עבר",
      "אושר",
      "מאושר",
      "אומת",
      "מאומת",
      "תוקף",
      "הצליח"
    ]
  },
  "am": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "አለፈ",
      "ጸደቀ",
      "ጸድቋል",
      "ተረጋገጠ",
      "ተረጋግጧል",
      "ተሳካ",
      "ተሳክቷል"
    ]
  },
  "fa": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "تأییدشده",
      "تأیید‌شده",
      "تاییدشده",
      "تایید‌شده",
      "تایید شده",
      "قبول‌شده",
      "گذرانده",
      "پاس‌شده",
      "موفق",
      "موفقیت‌آمیز",
      "اعتبارسنجی‌شده",
      "تصویب‌شده",
      "وارسی‌شده",
      "پذیرفته‌شده",
      "معتبر"
    ]
  },
  "hi": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "उत्तीर्ण",
      "पास",
      "सफल",
      "स्वीकृत",
      "अनुमोदित",
      "सत्यापित",
      "प्रमाणित",
      "मान्य"
    ]
  },
  "bn": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "উত্তীর্ণ",
      "পাস",
      "সফল",
      "অনুমোদিত",
      "যাচাইকৃত",
      "প্রমাণিত",
      "বৈধ"
    ]
  },
  "ur": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "کامیاب",
      "پاس",
      "منظور شدہ",
      "تصدیق شدہ",
      "توثیق شدہ",
      "مصدقہ"
    ]
  },
  "pa": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "ਪਾਸ",
      "ਸਫਲ",
      "ਮਨਜ਼ੂਰ",
      "ਪ੍ਰਵਾਨਿਤ",
      "ਤਸਦੀਕਸ਼ੁਦਾ",
      "ਪ੍ਰਮਾਣਿਤ"
    ]
  },
  "gu": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "પાસ",
      "સફળ",
      "મંજૂર",
      "ચકાસાયેલ",
      "પ્રમાણિત",
      "માન્ય"
    ]
  },
  "mr": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "उत्तीर्ण",
      "पास",
      "यशस्वी",
      "मंजूर",
      "पडताळलेले",
      "प्रमाणित",
      "वैध"
    ]
  },
  "si": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "සමත්",
      "සාර්ථක",
      "අනුමත",
      "තහවුරු කළ",
      "සත්‍යාපිත",
      "වලංගු"
    ]
  },
  "ta": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "தேர்ச்சி",
      "தேர்ச்சியடைந்தது",
      "அங்கீகரிக்கப்பட்டது",
      "சரிபார்க்கப்பட்டது",
      "உறுதிப்படுத்தப்பட்டது",
      "வெற்றிகரமானது"
    ]
  },
  "te": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "ఉత్తీర్ణం",
      "ఉత్తీర్ణమైంది",
      "ఆమోదించబడింది",
      "ధృవీకరించబడింది",
      "నిర్ధారించబడింది",
      "విజయవంతం"
    ]
  },
  "kn": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "ಉತ್ತೀರ್ಣ",
      "ಉತ್ತೀರ್ಣವಾಗಿದೆ",
      "ಅನುಮೋದಿಸಲಾಗಿದೆ",
      "ಪರಿಶೀಲಿಸಲಾಗಿದೆ",
      "ದೃಢೀಕರಿಸಲಾಗಿದೆ",
      "ಯಶಸ್ವಿ"
    ]
  },
  "ml": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "വിജയിച്ചു",
      "ഉത്തീർണ്ണമായി",
      "അംഗീകരിച്ചു",
      "സ്ഥിരീകരിച്ചു",
      "സാധൂകരിച്ചു"
    ]
  },
  "tr": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "geçti",
      "onaylandı",
      "doğrulandı",
      "başarılı",
      "başarıldı"
    ]
  },
  "az": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "keçdi",
      "təsdiqləndi",
      "doğrulandı",
      "təsdiq edildi",
      "uğurlu"
    ]
  },
  "kk": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "өтті",
      "мақұлданды",
      "расталды",
      "тексерілді",
      "сәтті"
    ]
  },
  "zh-hant": {
    "strategy": "substring",
    "negators": "不未沒否",
    "words": [
      "通過",
      "核准",
      "批准",
      "驗證",
      "已驗證",
      "成功",
      "認證",
      "合格"
    ]
  },
  "ja": {
    "strategy": "substring",
    "negators": "不未非無",
    "words": [
      "合格",
      "承認",
      "検証済み",
      "認証",
      "成功",
      "可決",
      "検証完了",
      "通過",
      "クリア"
    ]
  },
  "ko": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "통과",
      "합격",
      "승인",
      "검증됨",
      "성공",
      "유효"
    ]
  },
  "vi": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Đạt",
      "Thành công",
      "Đã duyệt",
      "Đã xác minh",
      "Đã xác thực"
    ]
  },
  "id": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Lulus",
      "Berhasil",
      "Disetujui",
      "Terverifikasi",
      "Tervalidasi"
    ]
  },
  "ms": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Lulus",
      "Berjaya",
      "Diluluskan",
      "Disahkan"
    ]
  },
  "th": {
    "strategy": "substring",
    "negators": "ไม่",
    "words": [
      "ผ่าน",
      "สำเร็จ",
      "อนุมัติ",
      "ยืนยันแล้ว",
      "ตรวจสอบแล้ว"
    ]
  },
  "km": {
    "strategy": "substring",
    "negators": "មិន",
    "words": [
      "ជោគជ័យ",
      "អនុម័ត",
      "ផ្ទៀងផ្ទាត់",
      "បានបញ្ជាក់"
    ]
  },
  "my": {
    "strategy": "substring",
    "negators": "မ",
    "words": [
      "အောင်မြင်",
      "အောင်",
      "ခွင့်ပြု",
      "အတည်ပြု",
      "စိစစ်အတည်ပြု"
    ]
  },
  "sw": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Imefaulu",
      "Imepita",
      "Imeidhinishwa",
      "Imethibitishwa",
      "Imehalalishwa",
      "Imefanikiwa",
      "Imekubaliwa"
    ]
  },
  "ha": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "An tabbatar",
      "An amince",
      "Ya ci",
      "An samu nasara",
      "An inganta"
    ]
  },
  "yo": {
    "strategy": "boundary",
    "negators": "",
    "words": [
      "Ti kọjá",
      "Ti fọwọ́sí",
      "Ti fìdí múlẹ̀",
      "Ti yege",
      "Àṣeyọrí",
      "Ti jẹ́rìí sí"
    ]
  }
}
const GUARDED = new Set(Object.keys(PER_LOCALE))

const esc = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
const BOUNDARY = '[\p{L}\p{M}]'

function parsePo(text) {
  const values = []
  let cur = null
  for (const line of text.split(/\r?\n/)) {
    if (/^#/.test(line)) { cur = null; continue }
    const m = line.match(/^msgstr\s+"(.*)"\s*$/)
    if (m) { cur = { v: m[1] }; values.push(cur); continue }
    const c = line.match(/^"(.*)"\s*$/)
    if (c && cur) { cur.v += c[1]; continue }
    if (/^msgid/.test(line) || line.trim() === '') cur = null
  }
  return values
    .map((x) => x.v)
    .filter((v) => v && !/MIME-Version|Content-Type|X-Generator|POT-Creation-Date|Language:/.test(v))
}

function langOf(text, file) {
  const m = text.match(/Language:\s*([A-Za-z0-9-]+)/)
  return (m ? m[1] : file.slice(0, -3)).toLowerCase()
}

function hits(value, locale) {
  const out = []
  for (const w of UNIVERSAL) if (new RegExp('\\b' + esc(w) + '\\b').test(value)) out.push(w)
  const conf = PER_LOCALE[locale]
  if (!conf) return out
  if (conf.strategy === 'boundary') {
    for (const w of conf.words) {
      if (new RegExp('(?<!' + BOUNDARY + ')' + esc(w) + '(?!' + BOUNDARY + ')', 'u').test(value)) out.push(w)
    }
  } else if (conf.strategy === 'substring') {
    for (const w of conf.words) {
      let i = value.indexOf(w)
      while (i >= 0) {
        const prev = value[i - 1]
        if (!(prev && conf.negators.includes(prev))) { out.push(w); break }
        i = value.indexOf(w, i + 1)
      }
    }
  } else {
    throw new Error('unknown strategy for ' + locale + ': ' + conf.strategy)
  }
  return out
}

const violations = []
const unguarded = []
for (const f of readdirSync(DIR).filter((f) => f.endsWith('.po'))) {
  const text = readFileSync(DIR + '/' + f, 'utf8')
  const loc = langOf(text, f)
  if (!GUARDED.has(loc)) { unguarded.push(f); continue }
  for (const v of parsePo(text)) for (const w of hits(v, loc)) violations.push({ f, w, v })
}

if (unguarded.length) {
  console.log(PREFIX + ': unguarded locale(s) - add a verdict word-list before shipping: ' + unguarded.join(', '))
}
if (violations.length) {
  console.log(PREFIX + ': forbidden positive-verdict word in a translation value (the front restitutes a verdict, never asserts one):')
  for (const { f, w, v } of violations) console.log('  ' + f + ': "' + w + '"  in  "' + v + '"')
}
if (unguarded.length || violations.length) process.exit(1)
NODE

echo "check-i18n-verdict-cross-locale: clean (no verdict word in any locale's translations)"
