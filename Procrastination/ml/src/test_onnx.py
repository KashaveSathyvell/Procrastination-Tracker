import onnxruntime as rt
sess = rt.InferenceSession("models/baseline_model.onnx")
print("INPUTS:")
for i in sess.get_inputs():
    print(f"  name={i.name}, shape={i.shape}, type={i.type}")
print("OUTPUTS:")
for o in sess.get_outputs():
    print(f"  name={o.name}, shape={o.shape}, type={o.type}")