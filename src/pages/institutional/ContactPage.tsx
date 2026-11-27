import { Bug, Lightbulb, Mail } from "lucide-react";
import { useAppShell } from "@/app/AppShellContext";
import {
  InfoCard,
  InstitutionalShell,
  SoonButton,
} from "@/components/institutional/InstitutionalShell";
import { Button } from "@/components/ui/button";
import { FOLIO_LINKS, hasLink } from "@/content/folioMeta";
import { openExternalUrl, openFolioLink } from "@/lib/openExternal";

export function ContactPage() {
  const { setSection } = useAppShell();

  return (
    <InstitutionalShell>
      <InfoCard
        title="Contacto y sugerencias"
        description="¿Encontraste un problema, tenés una idea o querés comunicarte?"
      >
        <p>
          No enviamos tus documentos ni datos locales. Los reportes se abren en canales externos
          (GitHub o correo) para que revises el contenido antes de enviarlo.
        </p>
      </InfoCard>

      <InfoCard title="Reportar un error" description="Problemas al utilizar Folio.">
        <p>
          Idealmente incluye versión de Folio, versión de Windows, pasos para reproducir y qué
          esperabas ver. Nunca adjuntes facturas, bases de datos ni archivos personales.
        </p>
        {hasLink("githubIssues") ? (
          <Button type="button" onClick={() => void openFolioLink("githubIssues")}>
            <Bug className="h-4 w-4" />
            Abrir reporte en GitHub
          </Button>
        ) : (
          <SoonButton label="Reportar error" />
        )}
      </InfoCard>

      <InfoCard title="Sugerir una mejora" description="Ideas para nuevas funciones.">
        <p>
          Contá qué te gustaría que Folio hiciera, qué problema resolvería y cómo lo usás hoy.
        </p>
        {hasLink("githubDiscussions") ? (
          <Button type="button" variant="outline" onClick={() => void openFolioLink("githubDiscussions")}>
            <Lightbulb className="h-4 w-4" />
            Proponer en GitHub
          </Button>
        ) : hasLink("githubIssues") ? (
          <Button type="button" variant="outline" onClick={() => void openFolioLink("githubIssues")}>
            <Lightbulb className="h-4 w-4" />
            Proponer en GitHub
          </Button>
        ) : (
          <SoonButton label="Sugerir mejora" />
        )}
      </InfoCard>

      <InfoCard title="Contacto general" description="Consultas o comentarios.">
        {hasLink("email") ? (
          <Button
            type="button"
            variant="outline"
            onClick={() => void openExternalUrl(`mailto:${FOLIO_LINKS.email}`)}
          >
            <Mail className="h-4 w-4" />
            Escribir a {FOLIO_LINKS.email}
          </Button>
        ) : (
          <SoonButton label="Contacto por correo" />
        )}
        <div className="flex flex-wrap gap-2 pt-1">
          <Button type="button" variant="outline" onClick={() => setSection("help")}>
            Centro de ayuda
          </Button>
          <Button type="button" variant="outline" onClick={() => setSection("privacy")}>
            Privacidad
          </Button>
        </div>
      </InfoCard>
    </InstitutionalShell>
  );
}
