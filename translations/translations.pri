isEmpty(QMAKE_LRELEASE) {
    !exists($$QMAKE_LRELEASE) { QMAKE_LRELEASE = lrelease }
}

unix {
    # fix for ArchLinux
    DEP_CHECK = $$system(which lrelease)
    contains($$DEP_CHECK, "no lrelease in") {
        QMAKE_LRELEASE = lrelease-qt4
    }
}

TR_PATH=../../translations

TRANSLATIONS += $$TR_PATH/svgcleaner_cs.ts \
                $$TR_PATH/svgcleaner_ru.ts \
                $$TR_PATH/svgcleaner_uk.ts \
                $$TR_PATH/svgcleaner_de.ts

CODECFORTR = UTF-8

updateqm.input = TRANSLATIONS
updateqm.output = ${QMAKE_FILE_BASE}.qm
updateqm.commands = $$QMAKE_LRELEASE -silent ${QMAKE_FILE_IN} -qm $$DESTDIR/${QMAKE_FILE_BASE}.qm
updateqm.CONFIG += no_link target_predeps
QMAKE_EXTRA_COMPILERS += updateqm
