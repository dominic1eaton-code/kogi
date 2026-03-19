import { ComponentFixture, TestBed } from '@angular/core/testing';
import { HubCooperativesComponent } from './hub-cooperatives.component';

describe('HubCooperativesComponent', () => {
  let component: HubCooperativesComponent;
  let fixture: ComponentFixture<HubCooperativesComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HubCooperativesComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(HubCooperativesComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
