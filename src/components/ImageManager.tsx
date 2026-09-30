import { FormEvent, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AndroidImageManifest, InstalledImage } from "../types";

type Props = {
  images: InstalledImage[];
  onChanged: () => Promise<void>;
};

const defaultManifest: AndroidImageManifest = {
  id: "android-16-x86_64",
  name: "Android 16",
  androidVersion: "16",
  api: 36,
  architecture: "x86_64",
  imageType: "aosp",
  disk: "android16.qcow2",
  diskFormat: "qcow2",
  recommended: true,
  notes: ""
};

export default function ImageManager({ images, onChanged }: Props) {
  const [manifest, setManifest] = useState(defaultManifest);
  const [sourceDisk, setSourceDisk] = useState("");
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);

  const removeImage = async (id: string, name: string) => {
    if (!window.confirm(`Remove ${name} from NekoDroid? This deletes its copied image files.`)) return;
    setBusy(true);
    try {
      await invoke("remove_android_image", { id });
      setOutput(`Removed ${name}.`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    try {
      const installed = await invoke<InstalledImage>("register_android_image", {
        manifest,
        sourceDisk
      });
      setOutput(`Registered ${installed.manifest.name} at ${installed.diskPath}`);
      await onChanged();
    } catch (error) {
      setOutput(String(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="panel">
      <div className="panel-heading">
        <div><p className="eyebrow">Android Images</p><h3>Installed image registry</h3></div>
        <span className="pill">{images.length} registered</span>
      </div>

      <div className="image-manager-grid">
        <div>
          {images.length === 0 ? (
            <p className="muted">No Android image has been registered yet.</p>
          ) : (
            <div className="image-list">
              {images.map(image => (
                <article className="instance-card" key={image.manifest.id}>
                  <div className="instance-title">
                    <strong>{image.manifest.name}</strong>
                    <span className={image.valid ? "status status-running" : "status status-error"}>
                      {image.valid ? "valid" : "invalid"}
                    </span>
                  </div>
                  <p>Android {image.manifest.androidVersion} · API {image.manifest.api} · {image.manifest.architecture}</p>
                  <small>{image.manifest.imageType} · {image.manifest.diskFormat}</small>
                  <small className="image-path">{image.diskPath}</small>
                  {image.validationError && <small>{image.validationError}</small>}
                  <div className="button-row">
                    <button
                      className="danger compact"
                      disabled={busy}
                      onClick={() => removeImage(image.manifest.id, image.manifest.name)}
                    >
                      Remove image
                    </button>
                  </div>
                </article>
              ))}
            </div>
          )}
        </div>

        <form className="image-register-form" onSubmit={submit}>
          <h4>Register local image</h4>
          <label>Image ID<input value={manifest.id} onChange={e => setManifest({...manifest,id:e.target.value})} /></label>
          <label>Name<input value={manifest.name} onChange={e => setManifest({...manifest,name:e.target.value})} /></label>
          <div className="split-fields">
            <label>Android<input value={manifest.androidVersion} onChange={e => setManifest({...manifest,androidVersion:e.target.value})} /></label>
            <label>API<input type="number" value={manifest.api} onChange={e => setManifest({...manifest,api:Number(e.target.value)})} /></label>
          </div>
          <div className="split-fields">
            <label>Type
              <select value={manifest.imageType} onChange={e => setManifest({...manifest,imageType:e.target.value})}>
                <option value="aosp">AOSP</option>
                <option value="gapps">GApps</option>
                <option value="microg">microG</option>
                <option value="custom">Custom</option>
              </select>
            </label>
            <label>Format
              <select value={manifest.diskFormat} onChange={e => setManifest({...manifest,diskFormat:e.target.value as AndroidImageManifest["diskFormat"],disk:e.target.value === "qcow2" ? "android.qcow2" : "android.img"})}>
                <option value="qcow2">QCOW2</option>
                <option value="raw">RAW/IMG</option>
              </select>
            </label>
          </div>
          <label>Stored disk filename<input value={manifest.disk} onChange={e => setManifest({...manifest,disk:e.target.value})} /></label>
          <label>Source disk path<input placeholder="C:\\Android\\android16.qcow2" value={sourceDisk} onChange={e => setSourceDisk(e.target.value)} /></label>
          <button className="primary" disabled={busy || !sourceDisk}>{busy ? "Copying..." : "Register Image"}</button>
          {output && <pre className="inline-output">{output}</pre>}
        </form>
      </div>
    </section>
  );
}
