// Theme: compact
// Dense layout for data-heavy specs: tighter margins, smaller text, tables pack more rows.
// Accent-teal palette. No watermark. Ideal for printing technical schedules.
#let accent  = rgb("#0d5c6b")
#let light   = rgb("#edf5f7")
#let muted   = rgb("#5a5a5a")

#let grid-images(..imgs) = {
  v(0.2em)
  align(center, block(width: 100%,
    grid(
      columns: (1fr, 1fr),
      gutter: 0.4em,
      ..imgs.pos().map(img =>
        block(stroke: 0.5pt + rgb("#cccccc"), radius: 2pt, clip: true)[#img]
      )
    )
  ))
  v(0.2em)
}

#let project-name = "Laem Chabang House"
#let build-date   = datetime.today().display("[day] [month repr:long] [year]")

#let conf(
  title: none,
  subtitle: none,
  authors: (),
  keywords: (),
  date: none,
  abstract-title: none,
  abstract: none,
  thanks: none,
  cols: 1,
  margin: (x: 1.4cm, top: 1.6cm, bottom: 2.0cm),
  paper: "a4",
  lang: "en",
  region: "US",
  font: none,
  fontsize: 9pt,
  mathfont: none,
  codefont: none,
  linestretch: 1.3,
  sectionnumbering: none,
  linkcolor: none,
  citecolor: none,
  filecolor: none,
  pagenumbering: "1",
  status: "Draft",
  rev: "1",
  doc,
) = {

  set document(
    title: if title != none { title } else { project-name },
    author: project-name,
  )

  set page(
    paper: paper,
    margin: margin,
    header: context {
      if counter(page).get().first() > 1 {
        set text(size: 7pt, fill: muted)
        grid(
          columns: (1fr, 1fr),
          align(left)[#project-name — #title],
          align(right)[#build-date],
        )
        line(length: 100%, stroke: 0.4pt + muted)
      }
    },
    footer: context {
      set text(size: 7pt, fill: muted)
      line(length: 100%, stroke: 0.4pt + muted)
      grid(
        columns: (1fr, 1fr, 1fr),
        align(left)[#title],
        align(center)[Rev #rev — #status],
        align(right)[#counter(page).display("1 / 1", both: true)],
      )
    },
  )

  set text(
    font: ("Inter", "Noto Sans", "Noto Sans Thai"),
    size: fontsize,
    lang: lang,
    hyphenate: false,
  )

  set par(leading: 0.5em, spacing: 0.75em)

  // Cover: compact side-by-side bar
  if title != none {
    block(width: 100%, fill: accent, inset: (x: 0.8em, y: 0.4em), radius: 3pt)[
      #grid(
        columns: (1fr, auto),
        align(left + horizon)[
          #text(size: 16pt, weight: "bold", fill: white)[#title]
        ],
        align(right + horizon)[
          #set text(size: 7pt, fill: white.transparentize(25%))
          #project-name · #build-date · Rev #rev — #status
        ],
      )
    ]
    v(0.5em)
  }

  // H1 — compact with light fill
  show heading.where(level: 1): it => {
    pagebreak(weak: true)
    v(0.4em)
    block(width: 100%, fill: light, inset: (x: 0.6em, y: 0.3em), radius: 2pt)[
      #text(size: 10.5pt, weight: "bold", fill: accent)[#it.body]
    ]
    v(0.2em)
  }

  // H2 — inline, no line
  show heading.where(level: 2): it => {
    v(0.35em)
    text(size: 9.5pt, weight: "bold", fill: accent)[#it.body]
    line(length: 100%, stroke: 1pt + accent)
    v(0.15em)
  }

  // H3
  show heading.where(level: 3): it => {
    v(0.3em)
    text(size: 9pt, weight: "bold", fill: muted)[#it.body]
    v(0.1em)
  }

  // Tables — tighter cell padding, more rows per page
  set table(
    inset: (x: 0.4em, y: 0.25em),
    stroke: (_, y) => if y == 0 { none } else { (top: 0.4pt + rgb("#dde8eb")) },
    fill: (_, y) => if y == 0 { accent } else if calc.odd(y) { rgb("#f4f9fa") } else { white },
  )
  show table.cell: set text(size: 7.5pt)
  show table.cell.where(y: 0): set text(weight: "bold", fill: white, size: 7.5pt)
  show table: set block(width: 100%)

  show link: set text(fill: accent)

  show figure: it => {
    v(0.2em)
    align(center)[
      #block(stroke: 0.5pt + rgb("#cccccc"), radius: 2pt, clip: true)[#it.body]
    ]
    v(0.2em)
  }

  show line: set line(stroke: 0.4pt + rgb("#cccccc"))

  doc
}
