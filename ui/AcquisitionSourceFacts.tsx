import { acquisitionSourceSummary } from "./acquisitionLabels";
import { useI18n } from "./i18n";
import type { AcquisitionSource, Catalog } from "./types";

/** Facts only; eligibility, clock scenario and held-item odds keep their own evidence. */
export function AcquisitionSourceFacts({
  source,
  catalog,
}: {
  source: AcquisitionSource;
  catalog: Catalog;
}) {
  const { t } = useI18n();
  return (
    <div className="acquisition-source-facts small">
      {acquisitionSourceSummary(source, catalog, t).map((line, i) => (
        <p key={i}>{line}</p>
      ))}
    </div>
  );
}
