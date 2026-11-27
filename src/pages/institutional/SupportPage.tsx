import { useAppShell } from "@/app/AppShellContext";
import {
  InfoCard,
  InstitutionalShell,
  SoonButton,
} from "@/components/institutional/InstitutionalShell";
import { Button } from "@/components/ui/button";
import mercadopagoLogo from "@/assets/brand/mercadopago.svg";
import { hasLink } from "@/content/folioMeta";
import { openFolioLink } from "@/lib/openExternal";

export function SupportPage() {
  const { setSection } = useAppShell();
  const canDonate = hasLink("mercadopago");

  return (
    <InstitutionalShell>
      <InfoCard title="¿Te sirve Folio?">
        <p>
          Folio nació con una idea simple: crear una herramienta útil, accesible y gratuita para que
          cualquier persona pueda organizar sus documentos sin complicaciones.
        </p>
        <p>
          No hay suscripciones, no hay funciones bloqueadas y no necesitás registrarte.
        </p>
        <p>
          Si Folio te ayuda en tu trabajo, en tu negocio o en tu vida cotidiana, podés colaborar
          voluntariamente para apoyar su desarrollo.
        </p>
        <p>
          Tu aporte ayuda a dedicar tiempo a nuevas funcionalidades, mejoras, pruebas y distribución.
        </p>
        <p className="font-medium text-foreground">
          No necesitás colaborar para utilizar Folio. Folio es y seguirá siendo gratuito.
        </p>
        <p>Gracias por acompañar el proyecto.</p>
      </InfoCard>

      <InfoCard
        title="Apoyar el desarrollo"
        description="Tu aporte ayuda a mantener, mejorar y distribuir Folio."
      >
        <div className="flex flex-wrap gap-2">
          {canDonate ? (
            <Button type="button" onClick={() => void openFolioLink("mercadopago")}>
              <img src={mercadopagoLogo} alt="" className="h-5 w-auto" />
              Apoyar con Mercado Pago
            </Button>
          ) : (
            <SoonButton label="Apoyar con Mercado Pago" />
          )}
        </div>
        <p className="text-xs">
          Se abre el enlace de Mercado Pago en el navegador. El importe lo elegís vos.
        </p>
        <Button type="button" variant="outline" onClick={() => setSection("about")}>
          Volver a Acerca de Folio
        </Button>
      </InfoCard>
    </InstitutionalShell>
  );
}
