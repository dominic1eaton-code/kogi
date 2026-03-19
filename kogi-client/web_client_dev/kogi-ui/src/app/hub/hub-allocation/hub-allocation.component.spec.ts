import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubAllocationComponent } from './hub-allocation.component';

describe('HubAllocationComponent', () => {
  let component: HubAllocationComponent;
  let fixture: ComponentFixture<HubAllocationComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubAllocationComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubAllocationComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
