import { useContext, useEffect, useState } from "react";
import { api } from "./api";
import { SearchSelect } from "./SearchSelect";
import { ConditionDetails, ConditionQueryRevision } from "./ConditionDetails";
import { useI18n } from "./i18n";
import type {
  AcquisitionSource,
  Catalog,
  MapFocus,
  QueryTarget,
} from "./types";

type Check = AcquisitionSource["conditions"][number];
type Service = {
  kind: string;
  minimum_level: number;
  choices: {
    menu_index: number;
    name: string;
    item: number;
    quantity: number;
    stat: number | null;
    mask: number;
    credit: Check;
    item_requirement: Check;
  }[];
  locations: {
    map_id: string;
    map_name: string;
    x: number;
    y: number;
    visibility: Check[];
  }[];
  text: string[];
  evidence: unknown;
};
type Report = { rom_md5: string; services: Service[]; partial: boolean };

export function TrainingServices({
  catalog,
  onMap,
  onTarget,
  onError,
}: {
  catalog: Catalog;
  onMap: (id: string, focus?: MapFocus) => void;
  onTarget: (target: QueryTarget) => void;
  onError: (error: unknown) => void;
}) {
  const revision = useContext(ConditionQueryRevision);
  const md5 = catalog.profile.md5;
  const [state, setState] = useState<{
    report: Report;
    revision: number;
  } | null>(null);
  useEffect(() => {
    let cancelled = false;
    setState(null);
    api<Report>("training_services", { expected_rom_md5: md5 })
      .then((report) => {
        if (!cancelled && report.rom_md5 === md5)
          setState({ report, revision });
      })
      .catch((error) => {
        if (!cancelled) onError(error);
      });
    return () => {
      cancelled = true;
    };
  }, [md5, revision]);
  const services =
    state?.revision === revision && state.report.rom_md5 === md5
      ? state.report.services
      : [];
  return services.map((service, index) => (
    <ServiceCard
      key={index}
      service={service}
      catalog={catalog}
      onMap={onMap}
      onTarget={onTarget}
    />
  ));
}
function ServiceCard({
  service,
  catalog,
  onMap,
  onTarget,
}: {
  service: Service;
  catalog: Catalog;
  onMap: (id: string, focus?: MapFocus) => void;
  onTarget: (target: QueryTarget) => void;
}) {
  const { t } = useI18n();
  const [selected, setSelected] = useState(0);
  const choice =
    service.choices.find((c) => c.menu_index === selected) ??
    service.choices[0];
  return (
    <article className="training-service">
      <h3>{t("trainingCrownService")}</h3>
      <p>
        {t("trainingCrownLevel")} {service.minimum_level}
      </p>
      <p className="small muted">{t("trainingCrownEffect")}</p>
      <p className="small muted">{t("trainingCrownAccess")}</p>
      {service.locations.map((location) => (
        <div key={`${location.map_id}:${location.x}:${location.y}`}>
          <button
            className="link-button"
            onClick={() =>
              onMap(location.map_id, { x: location.x, y: location.y })
            }
          >
            {location.map_name} · ({location.x}, {location.y}) ↗
          </button>
          <ConditionDetails
            checks={location.visibility}
            catalog={catalog}
            onMap={onMap}
            onTarget={onTarget}
          />
        </div>
      ))}
      <SearchSelect
        label={t("trainingCrownChoice")}
        value={choice?.menu_index ?? 0}
        options={service.choices.map((c) => ({
          value: c.menu_index,
          label: c.name,
        }))}
        onChange={(value) => setSelected(+value)}
      />
      {choice && (
        <section className="training-service-choice" key={choice.menu_index}>
          <strong>{choice.name}</strong>
          {" · "}
          <button
            className="link-button"
            onClick={() => onTarget({ kind: "item", id: choice.item })}
          >
            {catalog.items.find((item) => item.id === choice.item)?.name ??
              choice.item}{" "}
            × {choice.quantity} ↗
          </button>
          <p className="small">
            {t("trainingCrownCredit")} ·{" "}
            {choice.credit.actual ?? t("acqStatus_unknown")}
            {" · "}
            {t(
              choice.credit.satisfied === true
                ? "planConditionYes"
                : choice.credit.satisfied === false
                  ? "planConditionNo"
                  : "acqStatus_unknown",
            )}
          </p>
          <details>
            <summary>{t("trainingCrownConditions")}</summary>
            <ConditionDetails
              checks={[choice.credit, choice.item_requirement]}
              catalog={catalog}
              onMap={onMap}
              onTarget={onTarget}
            />
          </details>
        </section>
      )}
      <details>
        <summary>{t("trainingCrownDialogue")}</summary>
        {service.text.map((text, i) => (
          <p style={{ whiteSpace: "pre-wrap" }} key={i}>
            {text}
          </p>
        ))}
      </details>
      <details>
        <summary>{t("evidence")}</summary>
        <pre>{JSON.stringify(service, null, 2)}</pre>
      </details>
    </article>
  );
}
